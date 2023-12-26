use mlua::Lua;
use tokio::fs;

use crate::bindings::{Cast, Cha, UrlRef};

pub fn install(lua: &Lua) -> mlua::Result<()> {
	lua.globals().set(
		"fs",
		lua.create_table_from([
			(
				"write",
				lua.create_async_function(|_, (url, data): (UrlRef, mlua::String)| async move {
					Ok(fs::write(&*url, data).await.is_ok())
				})?,
			),
			(
				"metadata",
				lua.create_async_function(|lua, url: UrlRef| async move {
					fs::metadata(&*url).await.ok().map(|m| Cha::cast(lua, m)).transpose()
				})?,
			),
			(
				"symlink_metadata",
				lua.create_async_function(|lua, url: UrlRef| async move {
					fs::symlink_metadata(&*url).await.ok().map(|m| Cha::cast(lua, m)).transpose()
				})?,
			),
		])?,
	)
}
