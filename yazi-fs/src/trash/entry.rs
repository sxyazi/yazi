use std::{ffi::OsString, path::PathBuf};

use mlua::{IntoLua, Lua, Value};
use yazi_shared::path::PathBufDyn;

use crate::cha::Cha;

#[derive(Clone, Debug)]
pub struct TrashEntry {
	pub name:    OsString,
	pub key:     OsString,
	pub cha:     Cha,
	pub link_to: Option<PathBuf>,
	pub backing: PathBuf,
}

impl TrashEntry {
	#[cfg(all(unix, not(target_os = "android"), not(target_os = "ios")))]
	pub(super) fn new(path: PathBuf, name: OsString, key: OsString) -> std::io::Result<Self> {
		use super::TrashCha;

		let cha = Cha::from_trash(&path, &name, true)?;
		let link_to = if cha.is_link() { std::fs::read_link(&path).ok() } else { None };

		Ok(Self { name, key, cha, link_to, backing: path })
	}

	#[cfg(all(unix, not(target_os = "android"), not(target_os = "ios")))]
	pub(super) fn into_file(self, url: impl Into<yazi_shared::url::UrlBuf>) -> crate::file::File {
		use crate::file::{File, FileExtra};

		File {
			url:   url.into(),
			cha:   self.cha,
			extra: FileExtra::new(self.link_to.map(Into::into), Some(self.backing)),
		}
	}
}

impl IntoLua for TrashEntry {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let Self { name, key, cha, link_to, backing } = self;
		lua
			.create_table_from([
				("name", lua.create_external_string(name.into_encoded_bytes())?.into_lua(lua)?),
				("key", lua.create_external_string(key.into_encoded_bytes())?.into_lua(lua)?),
				("cha", cha.into_lua(lua)?),
				("link_to", link_to.map(PathBufDyn::Os).into_lua(lua)?),
				("backing", PathBufDyn::Os(backing).into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
