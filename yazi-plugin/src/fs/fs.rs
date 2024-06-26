use mlua::{IntoLuaMulti, Lua, Value};
use tokio::fs;

use crate::{bindings::Cast, cha::Cha, url::UrlRef};

pub fn install(lua: &Lua) -> mlua::Result<()> {
	lua.globals().raw_set(
		"fs",
		lua.create_table_from([
			(
				"write",
				lua.create_async_function(|lua, (url, data): (UrlRef, mlua::String)| async move {
					match fs::write(&*url, data).await {
						Ok(_) => (true, Value::Nil).into_lua_multi(lua),
						Err(e) => (false, e.raw_os_error()).into_lua_multi(lua),
					}
				})?,
			),
			(
				"cha",
				lua.create_async_function(|lua, url: UrlRef| async move {
					match fs::symlink_metadata(&*url).await {
						Ok(m) => (Cha::cast(lua, m)?, Value::Nil).into_lua_multi(lua),
						Err(e) => (Value::Nil, e.raw_os_error()).into_lua_multi(lua),
					}
				})?,
			),
			(
				"cha_follow",
				lua.create_async_function(|lua, url: UrlRef| async move {
					match fs::metadata(&*url).await {
						Ok(m) => (Cha::cast(lua, m)?, Value::Nil).into_lua_multi(lua),
						Err(e) => (Value::Nil, e.raw_os_error()).into_lua_multi(lua),
					}
				})?,
			),
		])?,
	)
}
