use anyhow::Context;
use futures::future::join_all;
use mlua::{ExternalError, ExternalResult, Function, IntoLuaMulti, Lua, LuaString, MultiValue, Table, Value, Variadic};
use tokio::{select, sync::mpsc};
use yazi_binding::{Handle, MpscRx, MpscTx, MpscUnboundedRx, MpscUnboundedTx, OneshotRx, OneshotTx, runtime, runtime_mut};
use yazi_core::{AppProxy, app::PluginOpt};
use yazi_runner::{RUNNER, loader::LOADER};
use yazi_shared::{LOCAL_SET, data::{Data, Sendable}};
use yazi_shim::{ResultExt, fs::Error, log::LOG_LEVEL};

use super::Utils;

impl Utils {
	pub(super) fn co(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, f: Function| {
			let thread = lua.create_thread(f)?;
			lua.create_async_function(move |lua, mut args: MultiValue| {
				let thread = thread.clone();
				async move {
					loop {
						let values: MultiValue = thread.resume(args)?;
						if let Some(Value::LightUserData(ud)) = values.front()
							&& *ud == Lua::poll_pending()
						{
							args = lua.yield_with(values).await?;
						} else {
							return Ok(values);
						}
					}
				}
			})
		})
	}

	pub(super) fn sync(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, f: Function| {
			let mut rt = runtime_mut!(lua)?;
			let Some(block) = rt.put_block(&f) else {
				return Err("`ya.sync()` must be called in a plugin").into_lua_err();
			};

			let current = rt.name().owned()?;
			lua.create_async_function(move |lua, mut args: MultiValue| {
				let (f, current) = (f.clone(), current.clone());
				async move {
					if runtime!(lua)?.is_blocking() {
						args.push_front(Value::Table(LOADER.try_load(&lua, &current)?));
						f.call::<MultiValue>(args)
					} else {
						Self::retrieve(&lua, &current, block, args)
							.await
							.and_then(|data| Sendable::list_to_values(&lua, data))
							.with_context(|| {
								format!("Failed to execute sync block-{block} in `{current}` plugin")
							})
							.into_lua_err()
					}
				}
			})
		})
	}

	pub(super) fn r#async(lua: &Lua, isolate: bool) -> mlua::Result<Function> {
		if isolate {
			lua.create_function(|_, _: Function| {
				Err::<(), _>("`ya.async()` can only be used in sync context at the moment".into_lua_err())
			})
		} else {
			lua.create_function(|lua, (f, args): (Function, MultiValue)| {
				let (name, scope) = runtime!(lua)?.name_scope()?;
				let lua = lua.clone();

				Ok(Handle::AsyncFn(LOCAL_SET.spawn_local(async move {
					runtime_mut!(lua)?.enter(&name, false, scope.clone());
					let result = select! {
						_ = scope.cancelled() => Ok(Default::default()),
						result = f.call_async(args) => result,
					};

					runtime_mut!(lua)?.leave()?;
					if let Err(ref e) = result {
						match name.as_str() {
							"init" => tracing::error!("Failed to execute async block in `init.lua`: {e}"),
							s => tracing::error!("Failed to execute async block in `{s}` plugin: {e}"),
						}
					}

					result
				})))
			})
		}
	}

	pub(super) fn async_blocking(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (f, arg): (Function, Value)| {
			let info = f.info();
			if info.what == "C" {
				return Err("`ya.async_blocking()` expects a Lua function".into_lua_err());
			}
			if info.num_upvalues > 1 || info.num_upvalues == 1 && f.environment().is_none() {
				return Err("`ya.async_blocking()` callback cannot capture local values".into_lua_err());
			}

			let (name, scope) = runtime!(lua)?.name_scope()?;
			let bytes = f.dump(LOG_LEVEL.get().is_none());
			let arg = Sendable::value_to_data(lua, arg)?;
			Ok(RUNNER.evaluate(name, scope, bytes, arg))
		})
	}

	pub(super) fn chan(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (r#type, buffer): (LuaString, Option<usize>)| {
			match (&*r#type.as_bytes(), buffer) {
				(b"mpsc", Some(buffer)) if buffer < 1 => {
					Err("Buffer size must be greater than 0".into_lua_err())
				}
				(b"mpsc", Some(buffer)) => {
					let (tx, rx) = tokio::sync::mpsc::channel::<Value>(buffer);
					(MpscTx::new(tx), MpscRx(rx)).into_lua_multi(lua)
				}
				(b"mpsc", None) => {
					let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Value>();
					(MpscUnboundedTx(tx), MpscUnboundedRx(rx)).into_lua_multi(lua)
				}
				(b"oneshot", _) => {
					let (tx, rx) = tokio::sync::oneshot::channel::<Value>();
					(OneshotTx(tx), OneshotRx(rx)).into_lua_multi(lua)
				}
				_ => Err("Channel type must be `mpsc` or `oneshot`".into_lua_err()),
			}
		})
	}

	pub(super) fn chunk(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, name: LuaString| async move {
			match LOADER.ensure(&name.to_str()?, |c| c.sync_peek).await {
				Ok(sync_peek) => lua.create_table_from([("sync_peek", sync_peek)])?.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::other(e.to_string())).into_lua_multi(&lua),
			}
		})
	}

	pub(super) fn join(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|_, fns: Variadic<Function>| async move {
			let mut results = MultiValue::with_capacity(fns.len());
			for r in join_all(fns.into_iter().map(|f| f.call_async::<MultiValue>(()))).await {
				results.extend(r?);
			}
			Ok(results)
		})
	}

	// TODO
	pub(super) fn select(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|_lua, _futs: MultiValue| async move { Ok(()) })
	}

	async fn retrieve(
		lua: &Lua,
		name: &str,
		calls: usize,
		args: MultiValue,
	) -> mlua::Result<Vec<Data>> {
		let args = Sendable::values_to_list(lua, args)?;
		let (tx, mut rx) = mpsc::channel::<Vec<Data>>(1);

		let name_ = name.to_owned();
		let callback = move |lua: &Lua, plugin: Table| {
			let Some(block) = runtime!(lua)?.get_block(&name_, calls) else {
				return Err("sync block not found".into_lua_err());
			};

			let args = [Ok(Value::Table(plugin))]
				.into_iter()
				.chain(args.into_iter().map(|d| Sendable::data_to_value(lua, d)))
				.collect::<mlua::Result<MultiValue>>()?;

			let values = Sendable::values_to_list(lua, block.call(args)?)?;
			tx.try_send(values).map_err(|_| "send failed".into_lua_err())
		};

		AppProxy::plugin(PluginOpt::new_callback(name.to_owned(), callback));

		rx.recv().await.ok_or("recv failed").into_lua_err()
	}
}
