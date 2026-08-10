use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_scheduler::custom::CustomOut;
use yazi_shared::{event::ActionCow, id::Id};

#[derive(Debug)]
pub struct OutputForm {
	pub id:  Id,
	pub out: CustomOut,
}

impl TryFrom<ActionCow> for OutputForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self {
			id:  a.get("id")?,
			out: a.take_any("out").ok_or_else(|| anyhow!("Invalid 'out' in OutputForm"))?,
		})
	}
}

impl FromLua for OutputForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for OutputForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
