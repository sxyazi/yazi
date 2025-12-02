use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
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

impl From<FilesOp> for UpdateFilesOpt {
	fn from(op: FilesOp) -> Self { Self { op } }
}

impl FromLua for UpdateFilesOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateFilesOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
