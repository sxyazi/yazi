use mlua::{ExternalError, Function, Lua, Table, Value, Variadic};
use tokio::sync::oneshot;
use yazi_shared::{emit, event::Cmd, Layer};

use super::Utils;
use crate::{OptData, ValueSendable};

impl Utils {
	pub(super) fn plugin(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.set(
			"plugin_retrieve",
			lua.create_async_function(
				|_, (name, blocks, args): (String, usize, Variadic<Value>)| async move {
					let args = Variadic::from_iter(ValueSendable::try_from_variadic(args)?);
					let (tx, rx) = oneshot::channel::<ValueSendable>();

					let data = OptData {
						init: Some(Box::new(move |lua| {
							let globals = lua.globals();
							globals.raw_set("YAZI_SYNC_BLOCKS", 0)?;
							globals.raw_set("YAZI_SYNC_CALLS", blocks)?;
							Ok(())
						})),
						cb: Some(Box::new(move |lua, _| {
							let globals = lua.globals();

							let entry = globals.raw_get::<_, Function>("YAZI_SYNC_ENTRY")?;
							globals.raw_set("YAZI_SYNC_ENTRY", Value::Nil)?;

							let value: ValueSendable = entry.call::<_, Value>(args)?.try_into()?;
							tx.send(value).map_err(|_| "send failed".into_lua_err())
						})),
						..Default::default()
					};

					emit!(Call(
						Cmd::args("plugin", vec![name.to_owned()]).with_bool("sync", true).with_data(data),
						Layer::App
					));

					rx.await
						.map_err(|_| format!("Failed to execute sync block in `{name}` plugin").into_lua_err())
				},
			)?,
		)?;

		Ok(())
	}
}
