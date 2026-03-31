use mlua::{FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VoidForm;

impl From<ActionCow> for VoidForm {
	fn from(_: ActionCow) -> Self { Self }
}

impl From<()> for VoidForm {
	fn from(_: ()) -> Self { Self }
}

impl FromLua for VoidForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Ok(Self) }
}

impl IntoLua for VoidForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.create_table()?.into_lua(lua) }
}
