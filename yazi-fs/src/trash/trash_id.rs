use std::{ffi::OsStr, io, path::{Path, PathBuf}};

use mlua::{BorrowedBytes, ExternalError, FromLua, Lua, Table, Value};
use yazi_shared::{path::PathBufDyn, strand::StrandLike};
use yazi_shim::{fs::Error, path::OsPathExt};

#[derive(Clone, Debug)]
pub struct TrashId {
	top: PathBuf,
	rel: PathBuf,
}

impl TrashId {
	pub(super) fn new<T, R>(top: T, rel: R) -> io::Result<Self>
	where
		T: Into<PathBuf>,
		R: Into<PathBuf>,
	{
		let top = top.into();
		let rel = rel.into();

		if top.as_os_str().is_empty() || !rel.is_joinable() {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid trash entry"));
		}
		if rel.has_parent_component() {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid trash entry path"));
		}

		Ok(Self { top, rel })
	}

	pub(super) fn top(&self) -> &Path { &self.top }

	pub(super) fn rel(&self) -> &Path { &self.rel }

	#[cfg(target_os = "macos")]
	pub(super) fn path(&self) -> PathBuf {
		if self.has_rel() { self.top.join(&self.rel) } else { self.top.clone() }
	}

	pub(super) fn child(&self, name: &OsStr) -> io::Result<Self> {
		Self::new(&self.top, self.rel.join(name))
	}

	pub(super) fn has_rel(&self) -> bool { !self.rel.as_os_str().is_empty() }
}

impl FromLua for TrashId {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;

		let top = t.raw_get::<BorrowedBytes>("top")?.to_os()?;
		let rel = t.raw_get::<PathBufDyn>("rel")?.into_os()?;
		Self::new(top, rel).map_err(|e| Error::from(e).into_lua_err())
	}
}
