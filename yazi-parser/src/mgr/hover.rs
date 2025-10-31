use std::path::PathBuf;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};

#[derive(Debug, Default)]
pub struct HoverOpt {
	pub urn: Option<PathBuf>,
}

impl From<Option<PathBuf>> for HoverOpt {
	fn from(urn: Option<PathBuf>) -> Self { Self { urn } }
}

impl FromLua for HoverOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for HoverOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
