use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;
use yazi_widgets::input::InputOpt;

#[derive(Debug, Default)]
pub struct ShowForm {
	pub opt: InputOpt,
}

impl From<InputOpt> for ShowForm {
	fn from(opt: InputOpt) -> Self { Self { opt } }
}

impl TryFrom<ActionCow> for ShowForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { opt: a.take_any("opt").ok_or_else(|| anyhow!("Invalid 'opt' in ShowForm"))? })
	}
}

impl FromLua for ShowForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ShowForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
