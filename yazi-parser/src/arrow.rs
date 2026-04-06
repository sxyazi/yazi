use mlua::{ExternalError, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::ActionCow;
use yazi_widgets::Step;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct ArrowForm {
	#[serde(alias = "0")]
	pub step: Step,
}

impl TryFrom<ActionCow> for ArrowForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl From<isize> for ArrowForm {
	fn from(n: isize) -> Self { Self { step: n.into() } }
}

impl IntoLua for ArrowForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
