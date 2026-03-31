use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_scheduler::TaskSummary;
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct UpdateProgressForm {
	pub summary: TaskSummary,
}

impl TryFrom<ActionCow> for UpdateProgressForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Some(summary) = a.take_any("summary") else {
			bail!("Invalid 'summary' in UpdateProgressForm");
		};

		Ok(Self { summary })
	}
}

impl FromLua for UpdateProgressForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateProgressForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
