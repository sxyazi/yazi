use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_fs::FilterCase;
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct FindForm {
	pub prev: bool,
	pub case: FilterCase,
}

impl TryFrom<ActionCow> for FindForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { prev: a.bool("previous"), case: FilterCase::from(&*a) })
	}
}

impl FromLua for FindForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for FindForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
