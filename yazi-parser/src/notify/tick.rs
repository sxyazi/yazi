use std::time::Duration;

use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct TickOpt {
	pub interval: Duration,
}

impl TryFrom<CmdCow> for TickOpt {
	type Error = anyhow::Error;

	fn try_from(c: CmdCow) -> Result<Self, Self::Error> {
		let Ok(interval) = c.first() else {
			bail!("Invalid 'interval' argument in TickOpt");
		};

		if interval < 0.0 {
			bail!("'interval' must be non-negative in TickOpt");
		}

		Ok(Self { interval: Duration::from_secs_f64(interval) })
	}
}

impl FromLua for TickOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TickOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
