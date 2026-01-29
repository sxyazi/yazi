use anyhow::bail;
use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;
use yazi_widgets::Step;

#[derive(Clone, Copy, Debug, Default)]
pub struct ArrowOpt {
	pub step: Step,
}

impl TryFrom<CmdCow> for ArrowOpt {
	type Error = anyhow::Error;

	fn try_from(c: CmdCow) -> Result<Self, Self::Error> {
		let Ok(step) = c.first() else {
			bail!("Invalid 'step' in ArrowOpt");
		};

		Ok(Self { step })
	}
}

impl From<isize> for ArrowOpt {
	fn from(n: isize) -> Self { Self { step: n.into() } }
}

impl IntoLua for ArrowOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
