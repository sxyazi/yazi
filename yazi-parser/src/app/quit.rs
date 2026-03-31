use mlua::{FromLua, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_core::app::QuitOpt;
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct QuitForm {
	pub opt: QuitOpt,
}

impl From<QuitOpt> for QuitForm {
	fn from(opt: QuitOpt) -> Self { Self { opt } }
}

impl TryFrom<ActionCow> for QuitForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { opt: if let Some(opt) = a.take_any("opt") { opt } else { a.try_into()? } })
	}
}

impl FromLua for QuitForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self { opt: <_>::from_lua(value, lua)? })
	}
}

impl IntoLua for QuitForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.opt.into_lua(lua) }
}
