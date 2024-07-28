use mlua::{ExternalError, ExternalResult, Function, IntoLua, Lua, MultiValue, Table, Value};
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
				if !rt.put_block(f.clone()) {
					return Err("`ya.sync()` must be called in a plugin").into_lua_err();
				}

				let cur = rt.current().unwrap().to_owned();
				lua.create_function(move |lua, args: MultiValue| {
					f.call::<_, MultiValue>(MultiValue::from_iter(
						[LOADER.load(lua, &cur)?.into_lua(lua)?].into_iter().chain(args),
					))
				})
			})?,
		)?;

		Ok(())
	}

	pub(super) fn sync_isolate(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set(
			"sync",
			lua.create_function(|lua, ()| {
				let Some(block) = lua.named_registry_value::<RtRef>("rt")?.next_block() else {
					return Err("`ya.sync()` must be called in a plugin").into_lua_err();
				};

				lua.create_async_function(move |lua, args: MultiValue| async move {
					if let Some(cur) = lua.named_registry_value::<RtRef>("rt")?.current() {
						Sendable::list_to_values(lua, Self::retrieve(cur, block, args).await?)
					} else {
						Err("block spawned by `ya.sync()` must be called in a plugin").into_lua_err()
					}
				})
			})?,
		)?;

		Ok(())
	}

	async fn retrieve(name: &str, calls: usize, args: MultiValue<'_>) -> mlua::Result<Vec<Data>> {
		let args = Sendable::values_to_vec(args)?;
		let (tx, rx) = oneshot::channel::<Vec<Data>>();

		let callback: OptCallback = {
			let name = name.to_owned();
			Box::new(move |lua, plugin| {
				let Some(block) = lua.named_registry_value::<RtRef>("rt")?.get_block(&name, calls) else {
					return Err("sync block not found".into_lua_err());
				};

				let args: Vec<_> = [Ok(Value::Table(plugin))]
					.into_iter()
					.chain(args.into_iter().map(|d| Sendable::data_to_value(lua, d)))
					.collect::<mlua::Result<_>>()?;

				let values = Sendable::values_to_vec(block.call(MultiValue::from_vec(args))?)?;
				tx.send(values).map_err(|_| "send failed".into_lua_err())
			})
		};

		emit!(Call(
			Cmd::args("plugin", vec![name.to_owned()])
				.with_bool("sync", true)
				.with_any("callback", callback),
			Layer::App
		));

		rx.await.map_err(|_| {
			format!("Failed to execute sync block-{calls} in `{name}` plugin").into_lua_err()
		})
	}
}
