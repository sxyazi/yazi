use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::path::PathBufDyn;

#[derive(Debug, Default)]
pub struct HoverForm {
	pub urn: Option<PathBufDyn>,
}

impl From<Option<PathBufDyn>> for HoverForm {
	fn from(urn: Option<PathBufDyn>) -> Self { Self { urn } }
}

impl FromLua for HoverForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for HoverForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
