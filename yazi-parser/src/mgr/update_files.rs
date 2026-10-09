use anyhow::bail;
use mlua::{FromLua, IntoLua, Lua, Table, Value};
use yazi_fs::op::FilesOp;
use yazi_shared::{event::{ActionCow, FromAction}, id::Id};

#[derive(Debug)]
pub struct UpdateFilesForm {
	pub op:   FilesOp,
	pub tabs: Vec<Id>,
}

impl<C> FromAction<C> for UpdateFilesForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		let Some(op) = a.take_any("op") else {
			bail!("Invalid 'op' in UpdateFilesForm");
		};

		Ok(Self { op, tabs: vec![] })
	}
}

impl From<FilesOp> for UpdateFilesForm {
	fn from(op: FilesOp) -> Self { Self { op, tabs: vec![] } }
}

impl FromLua for UpdateFilesForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;

		Ok(Self { op: t.raw_get("op")?, tabs: t.raw_get("tabs")? })
	}
}

impl IntoLua for UpdateFilesForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([("op", self.op.into_lua(lua)?), ("tabs", self.tabs.into_lua(lua)?)])?
			.into_lua(lua)
	}
}
