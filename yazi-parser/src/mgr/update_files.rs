use anyhow::bail;
use mlua::{FromLua, IntoLua, Lua, Table, Value};
use yazi_fs::FilesOp;
use yazi_shared::{event::ActionCow, id::Id};

#[derive(Debug)]
pub struct UpdateFilesForm {
	pub op:   FilesOp,
	pub tabs: Vec<Id>,
}

impl TryFrom<ActionCow> for UpdateFilesForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
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
