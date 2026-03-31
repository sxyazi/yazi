use std::time::Duration;

use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug, Default)]
pub struct TickForm {
	pub interval: Duration,
}

impl TryFrom<ActionCow> for TickForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> {
		let Ok(interval) = a.first() else {
			bail!("Invalid 'interval' in TickForm");
		};

		if interval < 0.0 {
			bail!("'interval' must be non-negative in TickForm");
		}

		Ok(Self { interval: Duration::from_secs_f64(interval) })
	}
}

impl FromLua for TickForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TickForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
