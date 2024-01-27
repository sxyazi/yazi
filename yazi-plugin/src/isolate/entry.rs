use mlua::{ExternalError, ExternalResult, IntoLua, Table, TableExt, Variadic};
use tokio::runtime::Handle;

use super::slim_lua;
use crate::{ValueSendable, LOADED};

pub async fn entry(name: String, args: Vec<ValueSendable>) -> mlua::Result<()> {
	LOADED.ensure(&name).await.into_lua_err()?;

	tokio::task::spawn_blocking(move || {
		let lua = slim_lua()?;
		let args = Variadic::from_iter(args.into_iter().filter_map(|v| v.into_lua(&lua).ok()));

		let plugin: Table = if let Some(b) = LOADED.read().get(&name) {
			lua.load(b).call(args)?
		} else {
			return Err("unloaded plugin".into_lua_err());
		};

		Handle::current().block_on(plugin.call_async_method("entry", ()))
	})
	.await
	.into_lua_err()?
}
