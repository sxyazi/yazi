use anyhow::bail;
use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;
use yazi_widgets::Step;

#[derive(Clone, Copy, Debug, Default)]
pub struct ArrowForm {
	pub step: Step,
}

impl TryFrom<ActionCow> for ArrowForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> {
		let Ok(step) = a.first() else {
			bail!("Invalid 'step' in ArrowForm");
		};

		Ok(Self { step })
	}
}

impl From<isize> for ArrowForm {
	fn from(n: isize) -> Self { Self { step: n.into() } }
}

impl IntoLua for ArrowForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
