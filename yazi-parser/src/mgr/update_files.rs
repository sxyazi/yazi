use anyhow::bail;
use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_fs::FilesOp;
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct UpdateFilesOpt {
	pub op: FilesOp,
}

impl TryFrom<CmdCow> for UpdateFilesOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(op) = c.take_any("op") else {
			bail!("Invalid 'op' argument in UpdateFilesOpt");
		};

		Ok(Self { op })
	}
}

impl IntoLua for &UpdateFilesOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
