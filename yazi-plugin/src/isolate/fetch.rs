use mlua::{ExternalError, ExternalResult, Table, TableExt};
use tokio::runtime::Handle;
use yazi_config::LAYOUT;

use super::slim_lua;
use crate::{bindings::Cast, elements::Rect, file::File, loader::LOADER};

pub async fn fetch(name: &str, files: Vec<yazi_shared::fs::File>) -> mlua::Result<u8> {
	LOADER.ensure(name).await.into_lua_err()?;

	let name = name.to_owned();
	tokio::task::spawn_blocking(move || {
		let lua = slim_lua(&name)?;
		let plugin: Table = if let Some(b) = LOADER.read().get(&name) {
			lua.load(b.as_ref()).set_name(name).call(())?
		} else {
			return Err("unloaded plugin".into_lua_err());
		};

		let files = files.into_iter().filter_map(|f| File::cast(&lua, f).ok()).collect::<Vec<_>>();
		if files.is_empty() {
			return Err("no files".into_lua_err());
		}

		plugin.raw_set("skip", 0)?;
		plugin.raw_set("area", Rect::cast(&lua, LAYOUT.load().preview)?)?;
		plugin.raw_set("files", files)?;

		Handle::current().block_on(plugin.call_async_method("fetch", ()))
	})
	.await
	.into_lua_err()?
}
