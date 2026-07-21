use std::{ffi::OsString, io, path::{Component, PathBuf}};

use mlua::{BorrowedBytes, FromLua, IntoLua, Lua, Table, Value};
use yazi_shared::{path::PathBufDyn, strand::AsStrand};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrashNode {
	pub(super) key: OsString,
	pub(super) top: OsString,
	pub(super) rel: PathBuf,
}

impl TrashNode {
	pub(super) fn validate(&self) -> io::Result<()> {
		if self.key.is_empty() || self.top.is_empty() || !self.rel.is_relative() {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid trash node"));
		}
		if self.rel.components().any(|c| matches!(c, Component::ParentDir)) {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid trash node path"));
		}
		Ok(())
	}
}

impl FromLua for TrashNode {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;
		let node = Self {
			key: t.raw_get::<BorrowedBytes>("key")?.as_strand().to_os_string()?,
			top: t.raw_get::<BorrowedBytes>("top")?.as_strand().to_os_string()?,
			rel: t.raw_get::<PathBufDyn>("rel")?.into_os()?,
		};

		node.validate().map_err(mlua::Error::external)?;
		Ok(node)
	}
}

impl IntoLua for TrashNode {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("key", lua.create_external_string(self.key.into_encoded_bytes())?.into_lua(lua)?),
				("top", lua.create_external_string(self.top.into_encoded_bytes())?.into_lua(lua)?),
				("rel", PathBufDyn::from(self.rel).into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
