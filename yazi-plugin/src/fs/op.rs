use mlua::{Table, UserData};
use yazi_codegen::FromLuaOwned;
use yazi_macro::impl_data_any;

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
	pub(super) fn part(t: Table) -> mlua::Result<Self> {
		let id = t.raw_get("id")?;
		let url = t.raw_get("url")?;
		let files = t.raw_get("files")?;

		Ok(Self(yazi_fs::op::FilesOp::Part(url, files, id)))
	}

	pub(super) fn done(t: Table) -> mlua::Result<Self> {
		let id = t.raw_get("id")?;
		let file = t.raw_get("file")?;

		Ok(Self(yazi_fs::op::FilesOp::Done(file, id)))
	}

	pub(super) fn size(t: Table) -> mlua::Result<Self> {
		let url = t.raw_get("url")?;
		let sizes: mlua::Result<_> = t.raw_get::<Table>("sizes")?.pairs().collect();

		Ok(Self(yazi_fs::op::FilesOp::Size(url, sizes?)))
	}

	pub(super) fn rank(t: Table) -> mlua::Result<Self> {
		let url = t.raw_get("url")?;
		let ranks: mlua::Result<_> = t.raw_get::<Table>("ranks")?.pairs().collect();

		Ok(Self(yazi_fs::op::FilesOp::Rank(url, ranks?)))
	}

	pub(super) fn upsert(t: Table) -> mlua::Result<Self> {
		let url = t.raw_get("url")?;
		let files = t.raw_get::<Table>("files")?.pairs().collect::<mlua::Result<_>>()?;

		Ok(Self(yazi_fs::op::FilesOp::Upsert(url, files)))
	}
}
