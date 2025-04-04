use mlua::{IntoLua, Lua, Table};
use yazi_binding::Url;

use crate::{Id, bindings::Cha, file::File};

pub(super) struct FilesOp(yazi_fs::FilesOp);

impl FilesOp {
	pub(super) fn part(_: &Lua, t: Table) -> mlua::Result<Self> {
		let id: Id = t.raw_get("id")?;
		let url: Url = t.raw_get("url")?;
		let files: Table = t.raw_get("files")?;

		Ok(Self(yazi_fs::FilesOp::Part(
			url.into(),
			files
				.sequence_values::<File>()
				.map(|f| f.map(Into::into))
				.collect::<mlua::Result<Vec<_>>>()?,
			*id,
		)))
	}

	pub(super) fn done(_: &Lua, t: Table) -> mlua::Result<Self> {
		let id: Id = t.raw_get("id")?;
		let cha: Cha = t.raw_get("cha")?;
		let url: Url = t.raw_get("url")?;

		Ok(Self(yazi_fs::FilesOp::Done(url.into(), *cha, *id)))
	}
}

impl IntoLua for FilesOp {
	fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
		lua.create_any_userdata(self.0)?.into_lua(lua)
	}
}
