use std::sync::Arc;

use anyhow::Context;
use futures::future::{join_all, select_all};
use mlua::{BorrowedBytes, ExternalError, ExternalResult, FromLua, Function, IntoLuaMulti, Lua, LuaString, MultiValue, ObjectLike, Value, Variadic};
use tokio::{select, sync::mpsc, task};
use yazi_binding::{Handle, MpscRx, MpscTx, MpscUnboundedRx, MpscUnboundedTx, OneshotRx, OneshotTx, runtime, runtime::RuntimeCo, runtime_mut};
use yazi_core::{AppProxy, app::PluginOpt};
use yazi_macro::log_if_err;
use yazi_runner::{CoHandle, RUNNER, loader::LOADER, sync::SyncJob};
use yazi_shared::{data::Data, sendable::Sendable};
use yazi_shim::{fs::Error, log::LOG_LEVEL};

use super::Utils;

impl Utils {
	pub(super) fn co(lua: &Lua) -> mlua::Result<Function> { lua.create_function(CoHandle::create) }

	pub(super) fn sync(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, f: Function| {
			let mut rt = runtime_mut!(lua)?;
			let Some(block) = rt.put_block(&f) else {
				return Err("`ya.sync()` must be called in a plugin").into_lua_err();
			};

			let current: Arc<str> = rt.name()?.into();
			lua.create_async_function(move |lua, mut args: MultiValue| {
				let (f, current) = (f.clone(), current.clone());
				async move {
					if runtime!(lua)?.is_blocking() {
						args.push_front(Value::Table(LOADER.try_load(&lua, &current)?));
						f.call::<MultiValue>(args)
					} else {
						Self::retrieve(&lua, &current, block, args)
							.await
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
		lua.create_function(move |lua, (f, args): (Function, MultiValue)| {
			if isolate {
				return Err("`ya.async()` can only be used in sync context at the moment".into_lua_err());
			}

			let lua = lua.clone();
			let seed = runtime!(lua)?.child_seed()?;

			Ok(Handle::AsyncFn(task::spawn_local(async move {
				let mut fut = RuntimeCo::new(&seed, lua, f.call_async(args));

				let result = select! {
					_ = seed.scope.cancelled() => Ok(Default::default()),
					r = &mut fut => r,
				};

				match seed.name.as_str() {
					"init" => log_if_err!("Async block in `init.lua`", &result),
					s => log_if_err!(&result, "Async block in `{s}` plugin",),
				}

				result
			})))
		})
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

			let seed = runtime!(lua)?.child_seed()?;
			let bytes = f.dump(LOG_LEVEL.get().is_none());
			let arg = Data::from_lua(arg, lua)?;
			Ok(RUNNER.evaluate(seed, bytes, arg))
		})
	}

	pub(super) fn chan(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (r#type, buffer): (BorrowedBytes, Option<usize>)| {
			match (&*r#type, buffer) {
				(b"mpsc", Some(0)) => Err("Buffer size must be greater than 0".into_lua_err()),
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

	pub(super) fn select(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, branches: Variadic<Value>| async move {
			if branches.is_empty() {
				return Err("`ya.select()` expects at least one callable value".into_lua_err());
			}

			let mut futures = Vec::with_capacity(branches.len());
			for branch in branches {
				let future = match branch {
					Value::Table(t) => t.call_async(()),
					Value::Function(f) => f.call_async(()),
					Value::UserData(ud) => ud.call_async(()),
					_ => return Err("`ya.select()` expects callable values".into_lua_err()),
				};

				futures.push(RuntimeCo::new(&*runtime!(lua)?, lua.clone(), future));
			}

			let (result, index, _) = select_all(futures).await;
			Ok((index + 1, result?))
		})
	}

	async fn retrieve(
		lua: &Lua,
		name: &str,
		block: usize,
		args: MultiValue,
	) -> mlua::Result<MultiValue> {
		let (tx, mut rx) = mpsc::channel(1);
		let args = Sendable::values_to_list(lua, args)?;

		let job = SyncJob {
			tab: runtime!(lua)?.tab(),
			name: name.to_owned().into(),
			block,
			args,
			scope: runtime!(lua)?.scope(),
		};
		let opt = PluginOpt::new_callback(job, move |lua, plugin, job| {
			let Some(block) = runtime!(lua)?.get_block(&job.name, job.block) else {
				return Err("sync block not found".into_lua_err());
			};

			let mut args = Sendable::list_to_values(lua, job.args)?;
			args.push_front(Value::Table(plugin));

			let values = Sendable::values_to_list(lua, block.call(args)?)?;
			tx.try_send(values).map_err(|_| "send failed".into_lua_err())
		});

		AppProxy::plugin(opt);

		let values = rx.recv().await.ok_or("recv failed").into_lua_err()?;
		Sendable::list_to_values(lua, values)
	}
}
