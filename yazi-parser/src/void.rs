use mlua::{FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VoidOpt;

impl From<CmdCow> for VoidOpt {
	fn from(_: CmdCow) -> Self { Self }
}

impl From<()> for VoidOpt {
	fn from(_: ()) -> Self { Self }
}

impl FromLua for VoidOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Ok(Self) }
}

impl IntoLua for VoidOpt {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.create_table()?.into_lua(lua) }
}
