use mlua::{FromLua, IntoLua, Lua, Value};
use yazi_core::which::WhichOpt;
use yazi_shared::{Layer, event::ActionCow};
use yazi_term::event::KeyEvent;

#[derive(Clone, Debug)]
pub struct ActivateForm {
	pub opt: WhichOpt,
}

impl From<(Layer, KeyEvent)> for ActivateForm {
	fn from(value: (Layer, KeyEvent)) -> Self { Self { opt: value.into() } }
}

impl TryFrom<ActionCow> for ActivateForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { opt: if let Some(opt) = a.take_any("opt") { opt } else { a.try_into()? } })
	}
}

impl FromLua for ActivateForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self { opt: WhichOpt::from_lua(value, lua)? })
	}
}

impl IntoLua for ActivateForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.opt.into_lua(lua) }
}
