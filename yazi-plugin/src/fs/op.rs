use mlua::{IntoLua, Lua, Table};
use yazi_binding::{Id, Url, Urn};

use crate::{bindings::Cha, file::File};

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

	pub(super) fn size(_: &Lua, t: Table) -> mlua::Result<Self> {
		let url: Url = t.raw_get("url")?;
		let sizes: Table = t.raw_get("sizes")?;

		Ok(Self(yazi_fs::FilesOp::Size(
			url.into(),
			sizes
				.pairs::<Urn, u64>()
				.map(|r| r.map(|(urn, size)| (urn.into(), size)))
				.collect::<mlua::Result<_>>()?,
		)))
	}
}

impl IntoLua for FilesOp {
	fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
		lua.create_any_userdata(self.0)?.into_lua(lua)
	}
}
