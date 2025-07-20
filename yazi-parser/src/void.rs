use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VoidOpt;

impl From<CmdCow> for VoidOpt {
	fn from(_: CmdCow) -> Self { Self }
}

impl From<()> for VoidOpt {
	fn from(_: ()) -> Self { Self }
}

impl IntoLua for VoidOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
