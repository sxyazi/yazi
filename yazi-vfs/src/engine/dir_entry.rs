use std::io;

use mlua::{UserData, UserDataFields, UserDataMethods};
use yazi_fs::{engine::FileHolder, file::File, stat::{Stat, StatType}};
use yazi_shared::{path::PathBufDyn, strand::{StrandCow, StrandLike}, url::UrlBuf};
use yazi_shim::{fs::Error, mlua::UserDataFieldsExt};

pub enum DirEntry {
	Local(yazi_fs::engine::local::DirEntry),
	Lua(super::lua::DirEntry),
	Sftp(super::sftp::DirEntry),
}

impl FileHolder for DirEntry {
	async fn file(&self) -> io::Result<File> {
		match self {
			Self::Local(dent) => dent.file().await,
			Self::Lua(dent) => dent.file().await,
			Self::Sftp(dent) => dent.file().await,
		}
	}

	async fn file_type(&self) -> io::Result<StatType> {
		match self {
			Self::Local(dent) => dent.file_type().await,
			Self::Lua(dent) => dent.file_type().await,
			Self::Sftp(dent) => dent.file_type().await,
		}
	}

	async fn lstat(&self) -> io::Result<Stat> {
		match self {
			Self::Local(dent) => dent.lstat().await,
			Self::Lua(dent) => dent.lstat().await,
			Self::Sftp(dent) => dent.lstat().await,
		}
	}

	fn name(&self) -> StrandCow<'_> {
		match self {
			Self::Local(dent) => dent.name(),
			Self::Lua(dent) => dent.name(),
			Self::Sftp(dent) => dent.name(),
		}
	}

	fn path(&self) -> PathBufDyn {
		match self {
			Self::Local(dent) => dent.path(),
			Self::Lua(dent) => dent.path(),
			Self::Sftp(dent) => dent.path(),
		}
	}

	fn url(&self) -> UrlBuf {
		match self {
			Self::Local(dent) => dent.url(),
			Self::Lua(dent) => dent.url(),
			Self::Sftp(dent) => dent.url(),
		}
	}
}

impl UserData for DirEntry {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("name", |lua, me| lua.create_string(me.name().encoded_bytes()));
		fields.add_cached_field("url", |_, me| Ok(me.url()));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method("dummy", |_, me, ()| async move {
			Ok(File::from_dummy(me.url(), me.file_type().await.ok()))
		});
		methods.add_async_method("lstat", |_, me, ()| async move {
			Ok(me.lstat().await.map_err(Error::from))
		});
		methods.add_async_method("file", |_, me, ()| async move {
			Ok(me.file().await.map_err(Error::from))
		});
	}
}
