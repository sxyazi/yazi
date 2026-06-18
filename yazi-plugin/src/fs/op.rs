use mlua::{Lua, Table, UserData};
use yazi_codegen::FromLuaOwned;
use yazi_fs::{cha::Cha, file::File};
use yazi_macro::impl_data_any;
use yazi_shared::{id::Id, path::PathBufDyn, url::UrlBuf};

#[derive(Clone, FromLuaOwned)]
pub(super) struct FilesOp(yazi_fs::FilesOp);

impl_data_any!(FilesOp => yazi_fs::FilesOp; from_into_lua = inherit);

impl From<FilesOp> for yazi_fs::FilesOp {
	fn from(op: FilesOp) -> Self { op.0 }
}

impl AsRef<yazi_fs::FilesOp> for FilesOp {
	fn as_ref(&self) -> &yazi_fs::FilesOp { &self.0 }
}

impl FilesOp {
	pub(super) fn part(_: &Lua, t: Table) -> mlua::Result<Self> {
		let id: Id = t.raw_get("id")?;
		let url: UrlBuf = t.raw_get("url")?;
		let files: Table = t.raw_get("files")?;

		Ok(Self(yazi_fs::FilesOp::Part(
			url.into(),
			files
				.sequence_values::<File>()
				.map(|f| f.map(Into::into))
				.collect::<mlua::Result<Vec<_>>>()?,
			id,
		)))
	}

	pub(super) fn done(_: &Lua, t: Table) -> mlua::Result<Self> {
		let id: Id = t.raw_get("id")?;
		let cha: Cha = t.raw_get("cha")?;
		let url: UrlBuf = t.raw_get("url")?;

		Ok(Self(yazi_fs::FilesOp::Done(url.into(), cha, id)))
	}

	pub(super) fn size(_: &Lua, t: Table) -> mlua::Result<Self> {
		let url: UrlBuf = t.raw_get("url")?;
		let sizes: Table = t.raw_get("sizes")?;

		Ok(Self(yazi_fs::FilesOp::Size(
			url.into(),
			sizes
				.pairs::<PathBufDyn, u64>()
				.map(|r| r.map(|(urn, size)| (urn.into(), size)))
				.collect::<mlua::Result<_>>()?,
		)))
	}
}

impl UserData for FilesOp {}
