use mlua::{ExternalError, ExternalResult, Function, IntoLua, Lua, Table, Value, Variadic};
use tokio::sync::oneshot;
use yazi_dds::Sendable;
use yazi_shared::{emit, event::{Cmd, Data}, Layer};

use super::Utils;
use crate::{loader::LOADER, runtime::RtRef, OptCallback};

impl Utils {
	pub(super) fn sync(lua: &'static Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set(
			"sync",
			lua.create_function(|lua, f: Function<'static>| {
				let mut rt = lua.named_registry_value::<RtRef>("rt")?;
				if !rt.push_block(f.clone()) {
					return Err("`ya.sync()` must be called in a plugin").into_lua_err();
				}

				let cur = rt.current.clone().unwrap();
				lua.create_function(move |lua, mut args: Variadic<Value>| {
					args.insert(0, LOADER.load(&cur)?.into_lua(lua)?);
					f.call::<_, Variadic<Value>>(args)
				})
			})?,
		)?;

		Ok(())
	}

	pub(super) fn sync_isolate(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set(
			"sync",
			lua.create_function(|lua, ()| {
				let block = lua.named_registry_value::<RtRef>("rt")?.next_block();
				lua.create_async_function(move |lua, args: Variadic<Value>| async move {
					let Some(cur) = lua.named_registry_value::<RtRef>("rt")?.current.clone() else {
						return Err("`ya.sync()` must be called in a plugin").into_lua_err();
					};

					Sendable::vec_to_variadic(lua, Self::retrieve(cur, block, args).await?)
				})
			})?,
		)?;

		Ok(())
	}

	async fn retrieve(
		name: String,
		calls: usize,
		args: Variadic<Value<'_>>,
	) -> mlua::Result<Vec<Data>> {
		let args = Sendable::variadic_to_vec(args)?;
		let (tx, rx) = oneshot::channel::<Vec<Data>>();

		let callback: OptCallback = {
			let name = name.clone();
			Box::new(move |lua, plugin| {
				let Some(block) = lua.named_registry_value::<RtRef>("rt")?.get_block(&name, calls) else {
					return Err("sync block not found".into_lua_err());
				};

				let mut self_args = Vec::with_capacity(args.len() + 1);
				self_args.push(Value::Table(plugin));
				for arg in args {
					self_args.push(Sendable::data_to_value(lua, arg)?);
				}

				let values = Sendable::variadic_to_vec(block.call(Variadic::from_iter(self_args))?)?;
				tx.send(values).map_err(|_| "send failed".into_lua_err())
			})
		};

		emit!(Call(
			Cmd::args("plugin", vec![name.clone()])
				.with_bool("sync", true)
				.with_any("callback", callback),
			Layer::App
		));

		rx.await.map_err(|_| {
			format!("Failed to execute sync block-{calls} in `{name}` plugin").into_lua_err()
		})
	}
}
