use mlua::{ExternalError, ExternalResult, Table, TableExt};
use tokio::runtime::Handle;

use super::slim_lua;
use crate::LOADED;

pub async fn entry(name: &str) -> mlua::Result<()> {
	LOADED.ensure(name).await.into_lua_err()?;

	let name = name.to_owned();
	tokio::task::spawn_blocking(move || {
		let lua = slim_lua()?;
		let plugin: Table = if let Some(b) = LOADED.read().get(&name) {
			lua.load(b).call(())?
		} else {
			return Err("unloaded plugin".into_lua_err());
		};

		Handle::current().block_on(plugin.call_async_method("entry", ()))
	})
	.await
	.into_lua_err()?
}
