use mlua::{ExternalError, ExternalResult, Table, TableExt};
use tokio::runtime::Handle;
use yazi_config::LAYOUT;

use super::slim_lua;
use crate::{bindings::{Cast, File}, elements::Rect, LOADED};

pub async fn preload(
	name: &str,
	files: Vec<yazi_shared::fs::File>,
	multi: bool,
) -> mlua::Result<u8> {
	LOADED.ensure(name).await.into_lua_err()?;

	let name = name.to_owned();
	tokio::task::spawn_blocking(move || {
		let lua = slim_lua()?;
		let plugin: Table = if let Some(b) = LOADED.read().get(&name) {
			lua.load(b).call(())?
		} else {
			return Err("unloaded plugin".into_lua_err());
		};

		let mut files = files.into_iter().filter_map(|f| File::cast(&lua, f).ok()).collect::<Vec<_>>();
		if files.is_empty() {
			return Err("no files".into_lua_err());
		}

		plugin.set("skip", 0)?;
		plugin.set("area", Rect::cast(&lua, LAYOUT.load().preview)?)?;
		if multi {
			plugin.set("files", files)?;
		} else {
			plugin.set("file", files.remove(0))?;
		}

		Handle::current().block_on(plugin.call_async_method("preload", ()))
	})
	.await
	.into_lua_err()?
}
