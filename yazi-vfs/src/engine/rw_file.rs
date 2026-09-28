use std::{io, pin::Pin};

use mlua::{IntoLuaMulti, LuaString, UserData, UserDataMethods, Value};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncWrite, AsyncWriteExt};
use yazi_fs::{engine::{Attrs, FileBuilder}, file::File};
use yazi_shared::url::{AsUrl, Url, UrlLike};
use yazi_shim::fs::Error;

use super::Demand;
use crate::VfsFile;

pub enum RwFile {
	Local(yazi_fs::engine::RwFile),
	Sftp(Box<super::sftp::RwFile>),
	Lua(super::lua::File),
}

impl From<yazi_fs::engine::RwFile> for RwFile {
	fn from(f: yazi_fs::engine::RwFile) -> Self { Self::Local(f) }
}

impl From<super::sftp::RwFile> for RwFile {
	fn from(f: super::sftp::RwFile) -> Self { Self::Sftp(Box::new(f)) }
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
	pub async fn create<U>(url: U) -> io::Result<Self>
	where
		U: AsUrl,
	{
		Demand::default().write(true).create(true).truncate(true).open(url).await
	}

	pub async fn create_new<U>(url: U) -> io::Result<Self>
	where
		U: AsUrl,
	{
		Demand::default().write(true).create_new(true).open(url).await
	}

	pub(crate) async fn metadata(&self) -> io::Result<yazi_fs::stat::Stat> {
		match self {
			Self::Local(f) => f.metadata().await,
			Self::Sftp(f) => f.metadata().await,
			Self::Lua(f) => f.metadata().await,
		}
	}

	pub async fn file(&self) -> io::Result<File> {
		match self {
			Self::Local(_) | Self::Sftp(..) => {
				Ok(File::from_follow(self.to_url(), self.metadata().await?).await)
			}
			Self::Lua(f) => f.file().await,
		}
	}

	pub async fn into_file(self) -> io::Result<File> {
		match self {
			Self::Local(f) => f.into_file().await,
			Self::Sftp(f) => f.into_file().await,
			Self::Lua(f) => f.into_file().await,
		}
	}

	pub(crate) fn seekless(&self) -> bool {
		match self {
			Self::Local(_) | Self::Sftp(_) => false,
			Self::Lua(f) => f.seekless,
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
		poll_rw!(self, poll_read, cx, buf)
	}
}

impl AsyncSeek for RwFile {
	#[inline]
	fn start_seek(mut self: Pin<&mut Self>, position: io::SeekFrom) -> io::Result<()> {
		poll_rw!(self, start_seek, position)
	}

	#[inline]
	fn poll_complete(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<io::Result<u64>> {
		poll_rw!(self, poll_complete, cx)
	}
}

impl AsyncWrite for RwFile {
	#[inline]
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &[u8],
	) -> std::task::Poll<Result<usize, io::Error>> {
		poll_rw!(self, poll_write, cx, buf)
	}

	#[inline]
	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), io::Error>> {
		poll_rw!(self, poll_flush, cx)
	}

	#[inline]
	fn poll_shutdown(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), io::Error>> {
		poll_rw!(self, poll_shutdown, cx)
	}

	#[inline]
	fn poll_write_vectored(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		bufs: &[io::IoSlice<'_>],
	) -> std::task::Poll<Result<usize, io::Error>> {
		poll_rw!(self, poll_write_vectored, cx, bufs)
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
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("flush", |lua, mut me, ()| async move {
			match me.flush().await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});
		methods.add_async_method_mut("read", |lua, mut me, len: usize| async move {
			let mut buf = vec![0; len];
			match me.read(&mut buf).await {
				Ok(n) => {
					buf.truncate(n);
					lua.create_external_string(buf)?.into_lua_multi(&lua)
				}
				Err(e) => (Value::Nil, Error::from(e)).into_lua_multi(&lua),
			}
		});
		methods.add_async_method_mut("write_all", |lua, mut me, src: LuaString| async move {
			match me.write_all(&src.as_bytes()).await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});
		methods.add_async_method_mut("shutdown", |lua, mut me, ()| async move {
			match me.shutdown().await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});
	}
}
