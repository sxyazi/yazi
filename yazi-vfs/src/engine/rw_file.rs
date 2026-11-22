use std::{io, pin::Pin, sync::Arc};

use mlua::{AnyUserData, BorrowedBytes, IntoLuaMulti, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRefMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt};
use yazi_binding::SeekFrom;
use yazi_fs::{engine::{Attrs, Random}, file::File};
use yazi_macro::impl_data_any;
use yazi_shared::{id::Id, url::{AsUrl, Url, UrlBuf, UrlLike}};
use yazi_shim::{fs::Error, mlua::UserDataFieldsExt};

use super::FileCursor;
use crate::VfsFile;

pub type RwFileRef = UserDataRefMut<RwFile>;

pub enum RwFile {
	Local(yazi_fs::engine::RwFile),
	Sftp(Pin<Box<super::sftp::RwFile>>),
	Lua(super::lua::File),
}

impl_data_any!(register Arc<RwFile>);

impl From<yazi_fs::engine::RwFile> for RwFile {
	fn from(f: yazi_fs::engine::RwFile) -> Self { Self::Local(f) }
}

impl From<super::sftp::RwFile> for RwFile {
	fn from(f: super::sftp::RwFile) -> Self { Self::Sftp(Box::pin(f)) }
}

impl From<super::lua::File> for RwFile {
	fn from(f: super::lua::File) -> Self { Self::Lua(f) }
}

impl AsUrl for RwFile {
	fn as_url(&self) -> Url<'_> {
		match self {
			Self::Local(f) => f.as_url(),
			Self::Sftp(f) => f.as_url(),
			Self::Lua(f) => f.as_url(),
		}
	}
}

impl UrlLike for RwFile {}

impl RwFile {
	async fn read_at(&self, offset: u64, len: usize) -> io::Result<Vec<u8>> {
		if !self.random().read() {
			return Err(io::ErrorKind::Unsupported.into());
		}

		let mut reader = FileCursor::reader(self, offset).await?;
		let mut buf = vec![0; len];

		let n = reader.read(&mut buf).await?;
		buf.truncate(n);

		Ok(buf)
	}

	async fn write_all_at(&self, offset: u64, src: &[u8]) -> io::Result<()> {
		if !self.random().write() {
			return Err(io::ErrorKind::Unsupported.into());
		}

		let mut writer = FileCursor::writer(self, offset).await?;
		let written = writer.write_all(src).await;

		let shutdown = if written.is_err() {
			// Discard unsent buffered bytes on failure, but still finish the underlying write stream.
			writer.get_mut().shutdown().await
		} else {
			writer.shutdown().await
		};

		written.and(shutdown)
	}

	pub(crate) async fn stat(&self) -> io::Result<yazi_fs::stat::Stat> {
		match self {
			Self::Local(f) => f.stat().await,
			Self::Sftp(f) => f.stat().await,
			Self::Lua(f) => f.stat().await,
		}
	}

	pub async fn file(&self) -> io::Result<File> {
		match self {
			Self::Local(_) | Self::Sftp(..) => {
				Ok(File::from_follow(self.to_url(), self.stat().await?).await)
			}
			Self::Lua(f) => f.file().await,
		}
	}

	pub fn into_url(self) -> UrlBuf {
		match self {
			Self::Local(f) => f.into_url(),
			Self::Sftp(f) => f.into_url(),
			Self::Lua(f) => f.into_url(),
		}
	}

	pub async fn into_file(self) -> io::Result<File> {
		match self {
			Self::Local(f) => f.into_file().await,
			Self::Sftp(f) => f.into_file().await,
			Self::Lua(f) => f.into_file().await,
		}
	}

	pub(crate) fn random(&self) -> Random {
		match self {
			Self::Local(_) | Self::Sftp(_) => Random::Both,
			Self::Lua(f) => f.random,
		}
	}

	pub(crate) async fn set_attrs(&self, attrs: Attrs) -> io::Result<()> {
		match self {
			Self::Local(f) => f.set_attrs(attrs).await,
			Self::Sftp(f) => f.set_attrs(attrs).await,
			Self::Lua(f) => f.set_attrs(attrs).await,
		}
	}

	pub(crate) async fn set_len(&self, size: u64) -> io::Result<()> {
		match self {
			Self::Local(f) => f.set_len(size).await,
			Self::Sftp(f) => f.set_len(size).await,
			Self::Lua(f) => f.set_len(size).await,
		}
	}
}

impl AsyncRead for RwFile {
	#[inline]
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &mut tokio::io::ReadBuf<'_>,
	) -> std::task::Poll<io::Result<()>> {
		poll_rw!(self, poll_read(cx, buf))
	}
}

impl AsyncSeek for RwFile {
	#[inline]
	fn start_seek(mut self: Pin<&mut Self>, position: io::SeekFrom) -> io::Result<()> {
		poll_rw!(self, start_seek(position))
	}

	#[inline]
	fn poll_complete(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<io::Result<u64>> {
		poll_rw!(self, poll_complete(cx))
	}
}

impl AsyncWrite for RwFile {
	#[inline]
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &[u8],
	) -> std::task::Poll<Result<usize, io::Error>> {
		poll_rw!(self, poll_write(cx, buf))
	}

	#[inline]
	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), io::Error>> {
		poll_rw!(self, poll_flush(cx))
	}

	#[inline]
	fn poll_shutdown(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), io::Error>> {
		poll_rw!(self, poll_shutdown(cx))
	}

	#[inline]
	fn poll_write_vectored(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		bufs: &[io::IoSlice<'_>],
	) -> std::task::Poll<Result<usize, io::Error>> {
		poll_rw!(self, poll_write_vectored(cx, bufs))
	}

	#[inline]
	fn is_write_vectored(&self) -> bool {
		match self {
			Self::Local(f) => f.is_write_vectored(),
			Self::Sftp(f) => f.is_write_vectored(),
			Self::Lua(f) => f.is_write_vectored(),
		}
	}
}

impl UserData for RwFile {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("url", |_, me| Ok(me.to_url()));
		fields.add_field_method_get("random", |_, me| Ok(me.random()));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_once("into_shared", |_, me, ()| Ok(Arc::new(me)));
		methods.add_async_method("stat", |_, me, ()| async move {
			Ok(me.stat().await.map_err(Error::from))
		});
		methods.add_async_method_mut("read", |lua, mut me, len: usize| async move {
			let mut buf = vec![0; len];
			Ok(match me.read(&mut buf).await {
				Ok(n) => {
					buf.truncate(n);
					Ok(lua.create_external_string(buf)?)
				}
				Err(e) => Err(Error::from(e)),
			})
		});
		methods.add_async_method("read_at", |lua, me, (offset, len): (Id, usize)| async move {
			Ok(match me.read_at(offset.get(), len).await {
				Ok(buf) => Ok(lua.create_external_string(buf)?),
				Err(e) => Err(Error::from(e)),
			})
		});
		methods.add_async_method_mut("seek", |_, mut me, from: SeekFrom| async move {
			Ok(me.seek(from.0).await.map_err(Error::from))
		});
		methods.add_async_method("set_attrs", |lua, me, attrs: Attrs| async move {
			match me.set_attrs(attrs).await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});
		methods.add_async_method("set_len", |lua, me, size: u64| async move {
			match me.set_len(size).await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});
		methods.add_async_method_mut("flush", |lua, mut me, ()| async move {
			match me.flush().await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});
		methods.add_async_method_mut("write_all", |lua, mut me, src: BorrowedBytes| async move {
			match me.write_all(&src).await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});
		methods.add_async_method(
			"write_all_at",
			|lua, me, (offset, src): (Id, BorrowedBytes)| async move {
				match me.write_all_at(offset.get(), &src).await {
					Ok(()) => true.into_lua_multi(&lua),
					Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
				}
			},
		);
		methods.add_async_method_mut("shutdown", |lua, mut me, ()| async move {
			match me.shutdown().await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});

		methods.add_meta_function(MetaMethod::Close, |_, ud: AnyUserData| ud.destroy());
	}
}
