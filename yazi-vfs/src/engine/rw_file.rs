use std::{io, pin::Pin};

use mlua::{IntoLuaMulti, LuaString, UserData, UserDataMethods, Value};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncWrite, AsyncWriteExt};
use yazi_binding::Error;
use yazi_fs::engine::Attrs;

pub enum RwFile {
	Tokio(tokio::fs::File),
	Sftp(Box<yazi_sftp::fs::File>),
	Lua(super::lua::File),
}

impl From<tokio::fs::File> for RwFile {
	fn from(f: tokio::fs::File) -> Self { Self::Tokio(f) }
}

impl From<yazi_sftp::fs::File> for RwFile {
	fn from(f: yazi_sftp::fs::File) -> Self { Self::Sftp(Box::new(f)) }
}

impl From<super::lua::File> for RwFile {
	fn from(f: super::lua::File) -> Self { Self::Lua(f) }
}

impl RwFile {
	// FIXME: path
	pub async fn metadata(&self) -> io::Result<yazi_fs::cha::Cha> {
		Ok(match self {
			Self::Tokio(f) => yazi_fs::cha::Cha::new("// FIXME", f.metadata().await?),
			Self::Sftp(f) => super::sftp::Cha::try_from(("// FIXME".as_bytes(), &f.fstat().await?))?.0,
			Self::Lua(f) => f.metadata().await?,
		})
	}

	pub async fn set_attrs(&self, attrs: Attrs) -> io::Result<()> {
		match self {
			Self::Tokio(f) => {
				let (perm, times) = (attrs.try_into(), attrs.try_into());
				if perm.is_err() && times.is_err() {
					return Ok(());
				}

				let std = f.try_clone().await?.into_std().await;
				tokio::task::spawn_blocking(move || {
					perm.map(|p| std.set_permissions(p)).ok();
					times.map(|t| std.set_times(t)).ok();
				})
				.await?;
			}
			Self::Sftp(f) => {
				if let Ok(attrs) = super::sftp::Attrs(attrs).try_into() {
					f.fsetstat(&attrs).await?;
				}
			}
			Self::Lua(f) => f.set_attrs(attrs).await?,
		}

		Ok(())
	}

	pub async fn set_len(&self, size: u64) -> io::Result<()> {
		Ok(match self {
			Self::Tokio(f) => f.set_len(size).await?,
			Self::Sftp(f) => {
				f.fsetstat(&yazi_sftp::fs::Attrs { size: Some(size), ..Default::default() }).await?
			}
			Self::Lua(f) => f.set_len(size).await?,
		})
	}
}

impl AsyncRead for RwFile {
	#[inline]
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &mut tokio::io::ReadBuf<'_>,
	) -> std::task::Poll<io::Result<()>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_read(cx, buf),
			Self::Sftp(f) => Pin::new(f).poll_read(cx, buf),
			Self::Lua(f) => Pin::new(f).poll_read(cx, buf),
		}
	}
}

impl AsyncSeek for RwFile {
	#[inline]
	fn start_seek(mut self: Pin<&mut Self>, position: io::SeekFrom) -> io::Result<()> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).start_seek(position),
			Self::Sftp(f) => Pin::new(f).start_seek(position),
			Self::Lua(f) => Pin::new(f).start_seek(position),
		}
	}

	#[inline]
	fn poll_complete(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<io::Result<u64>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_complete(cx),
			Self::Sftp(f) => Pin::new(f).poll_complete(cx),
			Self::Lua(f) => Pin::new(f).poll_complete(cx),
		}
	}
}

impl AsyncWrite for RwFile {
	#[inline]
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &[u8],
	) -> std::task::Poll<Result<usize, io::Error>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_write(cx, buf),
			Self::Sftp(f) => Pin::new(f).poll_write(cx, buf),
			Self::Lua(f) => Pin::new(f).poll_write(cx, buf),
		}
	}

	#[inline]
	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), io::Error>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_flush(cx),
			Self::Sftp(f) => Pin::new(f).poll_flush(cx),
			Self::Lua(f) => Pin::new(f).poll_flush(cx),
		}
	}

	#[inline]
	fn poll_shutdown(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), io::Error>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_shutdown(cx),
			Self::Sftp(f) => Pin::new(f).poll_shutdown(cx),
			Self::Lua(f) => Pin::new(f).poll_shutdown(cx),
		}
	}

	#[inline]
	fn poll_write_vectored(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		bufs: &[io::IoSlice<'_>],
	) -> std::task::Poll<Result<usize, io::Error>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_write_vectored(cx, bufs),
			Self::Sftp(f) => Pin::new(f).poll_write_vectored(cx, bufs),
			Self::Lua(f) => Pin::new(f).poll_write_vectored(cx, bufs),
		}
	}

	#[inline]
	fn is_write_vectored(&self) -> bool {
		match self {
			Self::Tokio(f) => f.is_write_vectored(),
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
				Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
			}
		});
		methods.add_async_method_mut("read", |lua, mut me, len: usize| async move {
			let mut buf = vec![0; len];
			match me.read(&mut buf).await {
				Ok(n) => {
					buf.truncate(n);
					lua.create_external_string(buf)?.into_lua_multi(&lua)
				}
				Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
			}
		});
		methods.add_async_method_mut("write_all", |lua, mut me, src: LuaString| async move {
			match me.write_all(&src.as_bytes()).await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
			}
		});
	}
}
