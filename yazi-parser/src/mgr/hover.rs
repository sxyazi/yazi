use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::url::UrnBuf;

#[derive(Debug, Default)]
pub struct HoverOpt {
	pub urn: Option<UrnBuf>,
}

impl From<Option<UrnBuf>> for HoverOpt {
	fn from(urn: Option<UrnBuf>) -> Self { Self { urn } }
}

impl FromLua for HoverOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for HoverOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
