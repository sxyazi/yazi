#[cfg(trash_unix)]
use std::ffi::OsString;
use std::{ffi::OsStr, fs, io, ops::Deref, path::{Path, PathBuf}};

use mlua::{AnyUserData, ExternalResult, FromLua, Lua, UserData, UserDataFields, UserDataMethods, Value};
use tokio::task::spawn_blocking;
use yazi_shared::{path::PathBufDyn, url::UrlBuf};
use yazi_shim::{fs::Error, mlua::UserDataFieldsExt};

use super::{TrashId, TrashStat};
use crate::{file::{File, FileExtra}, stat::{Stat, StatType}};

#[derive(Clone, Debug)]
pub struct TrashEntry {
	pub(super) id:       TrashId,
	pub(super) original: Option<PathBuf>,
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
		let backing = backing.into();

		id.rel()
			.file_name()
			.or_else(|| original.as_deref().and_then(Path::file_name))
			.or_else(|| backing.file_name())
			.filter(|name| !name.is_empty())
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid trash item path"))?;

		Ok(Self { id, original, backing })
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

	fn key(&self) -> &OsStr { self.rel().file_name().unwrap_or(self.top().as_os_str()) }

	fn name(&self) -> &OsStr {
		self
			.rel()
			.file_name()
			.or_else(|| self.original.as_deref().and_then(Path::file_name))
			.or_else(|| self.backing.file_name())
			.expect("trash entry must have a name")
	}

	pub(super) fn file_type(&self) -> io::Result<StatType> {
		Ok(fs::symlink_metadata(&self.backing)?.file_type().into())
	}

	pub(super) fn lstat(&self) -> io::Result<Stat> {
		Ok(Stat::new(self.name(), fs::symlink_metadata(&self.backing)?))
	}

	fn stat(&self) -> io::Result<Stat> {
		Stat::from_trash(&self.backing, self.name()).map(|(_, stat)| stat)
	}

	pub(super) fn file(&self, url: impl Into<UrlBuf>) -> io::Result<File> {
		let (lstat, stat) = Stat::from_trash(&self.backing, self.name())?;
		let link_to = if lstat.is_link() { fs::read_link(&self.backing).ok() } else { None };

		Ok(File {
			url: url.into(),
			stat,
			extra: FileExtra::new(lstat, link_to.map(Into::into), Some(self.backing.clone())),
		})
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
		fields.add_cached_field("original", |_, me| Ok(me.original.clone().map(PathBufDyn::Os)));
		fields.add_cached_field("backing", |_, me| Ok(PathBufDyn::Os(me.backing.clone())));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method("lstat", |_, me, ()| async move {
			let me = me.clone();

			Ok(spawn_blocking(move || me.lstat()).await.into_lua_err()?.map_err(Error::from))
		});
		methods.add_async_method("stat", |_, me, ()| async move {
			let me = me.clone();

			Ok(spawn_blocking(move || me.stat()).await.into_lua_err()?.map_err(Error::from))
		});
		methods.add_async_method("file", |_, me, url: UrlBuf| async move {
			let me = me.clone();

			Ok(spawn_blocking(move || me.file(url)).await.into_lua_err()?.map_err(Error::from))
		});
		methods.add_async_method("dummy", |_, me, url: UrlBuf| async move {
			let me = me.clone();

			spawn_blocking(move || File::from_dummy(url, me.file_type().ok())).await.into_lua_err()
		});
	}
}
