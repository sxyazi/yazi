use mlua::{Lua, Table, UserData};
use yazi_codegen::FromLuaOwned;
use yazi_fs::file::File;
use yazi_macro::impl_data_any;
use yazi_shared::{path::PathBufDyn, url::UrlBuf};

#[derive(Clone, FromLuaOwned, UserData)]
pub(super) struct FilesOp(yazi_fs::op::FilesOp);

impl_data_any!(FilesOp => yazi_fs::op::FilesOp; from_into_lua = inherit);

impl From<FilesOp> for yazi_fs::op::FilesOp {
	fn from(op: FilesOp) -> Self { op.0 }
}

impl AsRef<yazi_fs::op::FilesOp> for FilesOp {
	fn as_ref(&self) -> &yazi_fs::op::FilesOp { &self.0 }
}

impl FilesOp {
	pub(super) fn part(_: &Lua, t: Table) -> mlua::Result<Self> {
		let id = t.raw_get("id")?;
		let url = t.raw_get("url")?;
		let files: Table = t.raw_get("files")?;

		Ok(Self(yazi_fs::op::FilesOp::Part(
			url,
			files.sequence_values().collect::<mlua::Result<Vec<_>>>()?,
			id,
		)))
	}

	pub(super) fn done(_: &Lua, t: Table) -> mlua::Result<Self> {
		let id = t.raw_get("id")?;
		let file = t.raw_get("file")?;

		Ok(Self(yazi_fs::op::FilesOp::Done(file, id)))
	}

	pub(super) fn size(_: &Lua, t: Table) -> mlua::Result<Self> {
		let url = t.raw_get("url")?;
		let sizes: mlua::Result<_> = t.raw_get::<Table>("sizes")?.pairs().collect();

		Ok(Self(yazi_fs::op::FilesOp::Size(url, sizes?)))
	}

	pub(super) fn rank(_: &Lua, t: Table) -> mlua::Result<Self> {
		let url = t.raw_get("url")?;
		let ranks: mlua::Result<_> = t.raw_get::<Table>("ranks")?.pairs().collect();

		Ok(Self(yazi_fs::op::FilesOp::Rank(url, ranks?)))
	}

	pub(super) fn upsert(_: &Lua, t: Table) -> mlua::Result<Self> {
		let url: UrlBuf = t.raw_get("url")?;
		let files: Table = t.raw_get("files")?;
		let files = files.pairs::<PathBufDyn, File>().collect::<mlua::Result<_>>()?;

		Ok(Self(yazi_fs::op::FilesOp::Upsert(url, files)))
	}
}
