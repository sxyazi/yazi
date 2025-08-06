use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::CmdCow, url::Url};

#[derive(Debug)]
pub struct UpdateTasksOpt {
	pub urls: Vec<Url>,
}

impl TryFrom<CmdCow> for UpdateTasksOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(urls) = c.take_any("urls") else {
			bail!("Invalid 'urls' argument in UpdateTasksOpt");
		};

		Ok(Self { urls })
	}
}

impl FromLua for UpdateTasksOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateTasksOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
