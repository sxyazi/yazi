use std::{ffi::OsStr, ops::Deref, path::{Path, PathBuf}};
#[cfg(trash_unix)]
use std::{ffi::OsString, io};

use mlua::{AnyUserData, FromLua, Lua, UserData, UserDataFields, Value};
use yazi_shared::path::PathBufDyn;
use yazi_shim::mlua::UserDataFieldsExt;

use super::TrashId;
use crate::cha::Cha;

#[derive(Clone, Debug)]
pub(crate) struct TrashEntry {
	pub(super) id:       TrashId,
	pub(super) cha:      Cha,
	pub(super) lcha:     Cha,
	pub(super) original: Option<PathBuf>,
	pub(super) link_to:  Option<PathBuf>,
	pub(super) backing:  PathBuf,
}

impl Deref for TrashEntry {
	type Target = TrashId;

	fn deref(&self) -> &Self::Target { &self.id }
}

impl TrashEntry {
	#[cfg(trash_unix)]
	pub(super) fn new<B>(id: TrashId, backing: B, original: Option<PathBuf>) -> io::Result<Self>
	where
		B: Into<PathBuf>,
	{
		use super::TrashCha;
		let backing = backing.into();

		let name = id
			.rel()
			.file_name()
			.or_else(|| original.as_deref().and_then(Path::file_name))
			.or_else(|| backing.file_name())
			.filter(|name| !name.is_empty())
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid trash item path"))?;

		let (lcha, cha) = Cha::from_trash(&backing, name)?;
		let link_to = if lcha.is_link() { std::fs::read_link(&backing).ok() } else { None };

		Ok(Self { id, cha, lcha, original, link_to, backing })
	}

	#[cfg(trash_unix)]
	pub(super) fn top<T, B>(top: T, backing: B, original: Option<PathBuf>) -> io::Result<Self>
	where
		T: Into<PathBuf>,
		B: Into<PathBuf>,
	{
		Self::new(TrashId::new(top, PathBuf::new())?, backing, original)
	}

	#[cfg(trash_unix)]
	pub(super) fn child(&self, name: OsString) -> io::Result<Self> {
		let original = self.original.as_ref().map(|original| original.join(&name));
		Self::new(self.id.child(&name)?, self.backing.join(&name), original)
	}

	pub(super) fn key(&self) -> &OsStr { self.rel().file_name().unwrap_or(self.top().as_os_str()) }

	pub(super) fn name(&self) -> &OsStr {
		self
			.rel()
			.file_name()
			.or_else(|| self.original.as_deref().and_then(Path::file_name))
			.or_else(|| self.backing.file_name())
			.expect("trash entry must have a name")
	}

	#[cfg(trash_unix)]
	pub(super) fn into_file(self, url: impl Into<yazi_shared::url::UrlBuf>) -> crate::file::File {
		use crate::file::{File, FileExtra};

		File {
			url:   url.into(),
			cha:   self.cha,
			extra: FileExtra::new(self.link_to.map(Into::into), Some(self.backing)),
		}
	}
}

impl FromLua for TrashEntry {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		AnyUserData::from_lua(value, lua)?.take()
	}
}

impl UserData for TrashEntry {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("key", |lua, me| lua.create_string(me.key().as_encoded_bytes()));
		fields.add_cached_field("top", |lua, me| {
			lua.create_string(me.top().as_os_str().as_encoded_bytes())
		});
		fields.add_cached_field("rel", |_, me| Ok(PathBufDyn::from(me.rel())));
		fields.add_cached_field("name", |lua, me| lua.create_string(me.name().as_encoded_bytes()));
		fields.add_cached_field("cha", |_, me| Ok(me.cha));
		fields.add_cached_field("lcha", |_, me| Ok(me.lcha));
		fields.add_cached_field("original", |_, me| Ok(me.original.clone().map(PathBufDyn::Os)));
		fields.add_cached_field("link_to", |_, me| Ok(me.link_to.clone().map(PathBufDyn::Os)));
		fields.add_cached_field("backing", |_, me| Ok(PathBufDyn::Os(me.backing.clone())));
	}
}
