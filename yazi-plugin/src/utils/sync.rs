use mlua::{ExternalError, ExternalResult, Function, Lua, MultiValue, Value};
use tokio::sync::oneshot;
use yazi_dds::Sendable;
use yazi_proxy::{AppProxy, options::{PluginCallback, PluginOpt}};
use yazi_shared::event::Data;

use super::Utils;
use crate::{loader::LOADER, runtime::RtRef};

impl Utils {
	pub(super) fn sync(lua: &Lua, isolate: bool) -> mlua::Result<Function> {
		if isolate {
			lua.create_function(|lua, ()| {
				let Some(block) = lua.named_registry_value::<RtRef>("rt")?.next_block() else {
					return Err("`ya.sync()` must be called in a plugin").into_lua_err();
				};

				lua.create_async_function(move |lua, args: MultiValue| async move {
					if let Some(cur) = lua.named_registry_value::<RtRef>("rt")?.current() {
						Sendable::list_to_values(&lua, Self::retrieve(cur, block, args).await?)
					} else {
						Err("block spawned by `ya.sync()` must be called in a plugin").into_lua_err()
					}
				})
			})
		} else {
			lua.create_function(|lua, f: Function| {
				let mut rt = lua.named_registry_value::<RtRef>("rt")?;
				if !rt.put_block(f.clone()) {
					return Err("`ya.sync()` must be called in a plugin").into_lua_err();
				}

				let cur = rt.current().unwrap().to_owned();
				lua.create_function(move |lua, mut args: MultiValue| {
					args.push_front(Value::Table(LOADER.try_load(lua, &cur)?));
					f.call::<MultiValue>(args)
				})
			})
		}
	}

	async fn retrieve(id: &str, calls: usize, args: MultiValue) -> mlua::Result<Vec<Data>> {
		let args = Sendable::values_to_list(args)?;
		let (tx, rx) = oneshot::channel::<Vec<Data>>();

		let callback: PluginCallback = {
			let id = id.to_owned();
			Box::new(move |lua, plugin| {
				let Some(block) = lua.named_registry_value::<RtRef>("rt")?.get_block(&id, calls) else {
					return Err("sync block not found".into_lua_err());
				};

				let args = [Ok(Value::Table(plugin))]
					.into_iter()
					.chain(args.into_iter().map(|d| Sendable::data_to_value(lua, d)))
					.collect::<mlua::Result<_>>()?;

				let values = Sendable::values_to_list(block.call(MultiValue::from_vec(args))?)?;
				tx.send(values).map_err(|_| "send failed".into_lua_err())
			})
		};

		AppProxy::plugin(PluginOpt::new_callback(id, callback));

		rx.await
			.map_err(|_| format!("Failed to execute sync block-{calls} in `{id}` plugin").into_lua_err())
	}
}
