use mlua::{FromLua, IntoLua, Lua, Value};
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VoidForm;

impl<C> FromAction<C> for VoidForm {
	fn from_action(_: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(Self) }
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
