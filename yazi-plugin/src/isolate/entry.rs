use mlua::{ExternalError, ExternalResult, Table, TableExt};
use tokio::runtime::Handle;

use super::slim_lua;
use crate::{ValueSendable, LOADED};

pub async fn entry(name: String, args: Vec<ValueSendable>) -> mlua::Result<()> {
	LOADED.ensure(&name).await.into_lua_err()?;

	tokio::task::spawn_blocking(move || {
		let lua = slim_lua()?;
		lua.globals().set("YAZI_PLUGIN_NAME", lua.create_string(&name)?)?;

		let plugin: Table = if let Some(b) = LOADED.read().get(&name) {
			lua.load(b).call(())?
		} else {
			return Err("unloaded plugin".into_lua_err());
		};

		Handle::current().block_on(plugin.call_async_method("entry", args))
	})
	.await
	.into_lua_err()?
}
