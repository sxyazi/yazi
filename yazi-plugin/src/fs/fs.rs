use mlua::{IntoLua, Lua, Value};
use tokio::fs;

use crate::{bindings::{Cast, Cha}, url::UrlRef};

pub fn install(lua: &Lua) -> mlua::Result<()> {
	lua.globals().set(
		"fs",
		lua.create_table_from([
			(
				"write",
				lua.create_async_function(|lua, (url, data): (UrlRef, mlua::String)| async move {
					Ok(match fs::write(&*url, data).await {
						Ok(_) => (Value::Boolean(true), Value::Nil),
						Err(e) => (Value::Boolean(false), e.raw_os_error().into_lua(lua)?),
					})
				})?,
			),
			(
				"cha",
				lua.create_async_function(|lua, url: UrlRef| async move {
					Ok(match fs::symlink_metadata(&*url).await {
						Ok(m) => (Cha::cast(lua, m)?.into_lua(lua)?, Value::Nil),
						Err(e) => (Value::Nil, e.raw_os_error().into_lua(lua)?),
					})
				})?,
			),
			(
				"cha_follow",
				lua.create_async_function(|lua, url: UrlRef| async move {
					Ok(match fs::metadata(&*url).await {
						Ok(m) => (Cha::cast(lua, m)?.into_lua(lua)?, Value::Nil),
						Err(e) => (Value::Nil, e.raw_os_error().into_lua(lua)?),
					})
				})?,
			),
		])?,
	)
}
