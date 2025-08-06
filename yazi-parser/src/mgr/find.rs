use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_fs::FilterCase;
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct FindOpt {
	pub prev: bool,
	pub case: FilterCase,
}

impl TryFrom<CmdCow> for FindOpt {
	type Error = anyhow::Error;

	fn try_from(c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { prev: c.bool("previous"), case: FilterCase::from(&*c) })
	}
}

impl FromLua for FindOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for FindOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
