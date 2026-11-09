use std::io;

use mlua::{AnyUserData, IntoLuaMulti, MetaMethod, UserData, UserDataMethods, Value};
use yazi_fs::engine::DirReader;
use yazi_shim::fs::Error;

pub enum ReadDir {
	Local(yazi_fs::engine::local::ReadDir),
	Lua(super::lua::ReadDir),
	Sftp(super::sftp::ReadDir),
}

impl From<yazi_fs::engine::local::ReadDir> for ReadDir {
	fn from(reader: yazi_fs::engine::local::ReadDir) -> Self { Self::Local(reader) }
}

impl From<super::lua::ReadDir> for ReadDir {
	fn from(reader: super::lua::ReadDir) -> Self { Self::Lua(reader) }
}

impl From<super::sftp::ReadDir> for ReadDir {
	fn from(reader: super::sftp::ReadDir) -> Self { Self::Sftp(reader) }
}

impl DirReader for ReadDir {
	type Entry = super::DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> {
		Ok(match self {
			Self::Local(reader) => reader.next().await?.map(Self::Entry::Local),
			Self::Lua(reader) => reader.next().await?.map(Self::Entry::Lua),
			Self::Sftp(reader) => reader.next().await?.map(Self::Entry::Sftp),
		})
	}
}

impl UserData for ReadDir {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_meta_method_mut(MetaMethod::Call, |lua, mut me, ()| async move {
			match me.next().await {
				Ok(ent) => (ent, Value::Nil).into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});
		methods.add_meta_function(MetaMethod::Close, |_, ud: AnyUserData| ud.destroy());
	}
}
