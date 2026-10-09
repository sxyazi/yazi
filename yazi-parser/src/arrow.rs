use mlua::{ExternalError, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::{ActionCow, FromAction};
use yazi_widgets::Step;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct ArrowForm {
	#[serde(alias = "0")]
	pub step: Step,
}

impl<C> FromAction<C> for ArrowForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl From<isize> for ArrowForm {
	fn from(n: isize) -> Self { Self { step: n.into() } }
}

impl IntoLua for ArrowForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
