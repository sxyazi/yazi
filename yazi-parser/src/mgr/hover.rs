use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::path::PathBufDyn;

#[derive(Debug, Default)]
pub struct HoverOpt {
	pub urn: Option<PathBufDyn>,
}

impl From<Option<PathBufDyn>> for HoverOpt {
	fn from(urn: Option<PathBufDyn>) -> Self { Self { urn } }
}

impl FromLua for HoverOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for HoverOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
