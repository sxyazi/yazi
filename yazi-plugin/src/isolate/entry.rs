use mlua::{ExternalError, ExternalResult, Table, TableExt};
use tokio::runtime::Handle;
use yazi_dds::Sendable;
use yazi_shared::event::Data;

use super::slim_lua;
use crate::loader::LOADER;

pub async fn entry(name: String, args: Vec<Data>) -> mlua::Result<()> {
	LOADER.ensure(&name).await.into_lua_err()?;

	tokio::task::spawn_blocking(move || {
		let lua = slim_lua(&name)?;
		let plugin: Table = if let Some(b) = LOADER.read().get(&name) {
			lua.load(b.as_ref()).set_name(name).call(())?
		} else {
			return Err("unloaded plugin".into_lua_err());
		};

		Handle::current()
			.block_on(plugin.call_async_method("entry", Sendable::list_to_table(&lua, args)))
	})
	.await
	.into_lua_err()?
}
