use mlua::{ExternalError, ExternalResult, Function, IntoLua, Lua, Table, Value, Variadic};
use tokio::sync::oneshot;
use yazi_dds::ValueSendable;
use yazi_shared::{emit, event::Cmd, Layer};

use super::Utils;
use crate::{loader::LOADER, runtime::RtRef, OptData};

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
					if let Some(cur) = lua.named_registry_value::<RtRef>("rt")?.current.clone() {
						Self::retrieve(cur, block, args).await
					} else {
						Err("`ya.sync()` must be called in a plugin").into_lua_err()
					}
				})
			})?,
		)?;

		Ok(())
	}

	async fn retrieve(
		name: String,
		calls: usize,
		args: Variadic<Value<'_>>,
	) -> mlua::Result<mlua::Variadic<ValueSendable>> {
		let args = ValueSendable::try_from_variadic(args)?;
		let (tx, rx) = oneshot::channel::<Vec<ValueSendable>>();

		let data = OptData {
			cb: Some({
				let name = name.clone();
				Box::new(move |lua, plugin| {
					let Some(block) = lua.named_registry_value::<RtRef>("rt")?.get_block(&name, calls) else {
						return Err("sync block not found".into_lua_err());
					};

					let mut self_args = Vec::with_capacity(args.len() + 1);
					self_args.push(Value::Table(plugin));
					for arg in args {
						self_args.push(arg.into_lua(lua)?);
					}

					let values =
						ValueSendable::try_from_variadic(block.call(Variadic::from_iter(self_args))?)?;
					tx.send(values).map_err(|_| "send failed".into_lua_err())
				})
			}),
			..Default::default()
		};

		emit!(Call(
			Cmd::args("plugin", vec![name.clone()]).with_bool("sync", true).with_data(data),
			Layer::App
		));

		Ok(Variadic::from_iter(rx.await.map_err(|_| {
			format!("Failed to execute sync block-{calls} in `{name}` plugin").into_lua_err()
		})?))
	}
}
