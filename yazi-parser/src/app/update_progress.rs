use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use ordered_float::OrderedFloat;
use serde::Serialize;
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct UpdateProgressOpt {
	pub summary: TaskSummary,
}

impl TryFrom<CmdCow> for UpdateProgressOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(summary) = c.take_any("summary") else {
			bail!("Invalid 'summary' in UpdateProgressOpt");
		};

		Ok(Self { summary })
	}
}

impl FromLua for UpdateProgressOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateProgressOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

// --- Progress
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct TaskSummary {
	pub total:   u32,
	pub success: u32,
	pub failed:  u32,
	pub percent: Option<OrderedFloat<f32>>,
}
