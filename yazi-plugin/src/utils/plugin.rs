use mlua::{ExternalError, Function, IntoLua, Lua, Table, Value, Variadic};
use tokio::sync::oneshot;
use yazi_shared::{emit, event::Cmd, Layer};

use super::Utils;
use crate::{OptData, ValueSendable};

impl Utils {
	pub(super) fn plugin(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set(
			"plugin_retrieve",
			lua.create_async_function(
				|_, (name, calls, args): (String, usize, Variadic<Value>)| async move {
					let args = ValueSendable::try_from_variadic(args)?;
					let (tx, rx) = oneshot::channel::<Vec<ValueSendable>>();

					let data = OptData {
						cb: Some({
							let name = name.clone();
							Box::new(move |lua, plugin| {
								let blocks = lua.globals().raw_get::<_, Table>("YAZI_SYNC_BLOCKS")?;
								let block = blocks.raw_get::<_, Table>(name)?.raw_get::<_, Function>(calls)?;

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
				},
			)?,
		)?;

		Ok(())
	}
}
