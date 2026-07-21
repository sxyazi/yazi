use std::{io, pin::Pin};

use mlua::{IntoLuaMulti, LuaString, UserData, UserDataMethods, Value};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncWrite, AsyncWriteExt};
use yazi_binding::Error;
use yazi_fs::{engine::Attrs, file::File};
use yazi_shared::url::{UrlBuf, UrlLike};

use crate::VfsFile;

pub enum RwFile {
	Tokio(tokio::fs::File, UrlBuf),
	Sftp(Box<yazi_sftp::fs::File>, UrlBuf),
	Lua(super::lua::File),
}

impl From<(tokio::fs::File, UrlBuf)> for RwFile {
	fn from((f, url): (tokio::fs::File, UrlBuf)) -> Self { Self::Tokio(f, url) }
}

impl From<(yazi_sftp::fs::File, UrlBuf)> for RwFile {
	fn from((f, url): (yazi_sftp::fs::File, UrlBuf)) -> Self { Self::Sftp(Box::new(f), url) }
}

impl From<super::lua::File> for RwFile {
	fn from(f: super::lua::File) -> Self { Self::Lua(f) }
}

impl RwFile {
	pub async fn metadata(&self) -> io::Result<yazi_fs::cha::Cha> {
		Ok(match self {
			Self::Tokio(f, url) => {
				yazi_fs::cha::Cha::new(url.name().unwrap_or_default(), f.metadata().await?)
			}
			Self::Sftp(f, url) => {
				let name = url.name().unwrap_or_default().encoded_bytes();
				super::sftp::Cha::try_from((name, &f.fstat().await?))?.0
			}
			Self::Lua(f) => f.metadata().await?,
		})
	}

	pub async fn file(&self) -> io::Result<File> {
		Ok(match self {
			Self::Tokio(_, url) | Self::Sftp(_, url) => {
				let cha = self.metadata().await?;
				File::from_follow(url.clone(), cha).await
			}
			Self::Lua(f) => f.file().await?,
		})
	}

	pub async fn into_file(self) -> io::Result<File> {
		if let Self::Lua(f) = self {
			return f.into_file().await;
		}

		let cha = self.metadata().await?;
		Ok(match self {
			Self::Tokio(_, url) | Self::Sftp(_, url) => File { url, cha, extra: Default::default() },
			Self::Lua(_) => unreachable!(),
		})
	}

	pub async fn set_attrs(&self, attrs: Attrs) -> io::Result<()> {
		match self {
			Self::Tokio(f, _) => {
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
			Self::Sftp(f, _) => {
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
			Self::Tokio(f, _) => f.set_len(size).await?,
			Self::Sftp(f, _) => {
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
			RwFile::Tokio(f, _) => Pin::new(f).poll_read(cx, buf),
			RwFile::Sftp(f, _) => Pin::new(f).poll_read(cx, buf),
			RwFile::Lua(f) => Pin::new(f).poll_read(cx, buf),
		}
	}
}

impl AsyncSeek for RwFile {
	#[inline]
	fn start_seek(mut self: Pin<&mut Self>, position: io::SeekFrom) -> io::Result<()> {
		match &mut *self {
			RwFile::Tokio(f, _) => Pin::new(f).start_seek(position),
			RwFile::Sftp(f, _) => Pin::new(f).start_seek(position),
			RwFile::Lua(f) => Pin::new(f).start_seek(position),
		}
	}

	#[inline]
	fn poll_complete(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<io::Result<u64>> {
		match &mut *self {
			RwFile::Tokio(f, _) => Pin::new(f).poll_complete(cx),
			RwFile::Sftp(f, _) => Pin::new(f).poll_complete(cx),
			RwFile::Lua(f) => Pin::new(f).poll_complete(cx),
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
			RwFile::Tokio(f, _) => Pin::new(f).poll_write(cx, buf),
			RwFile::Sftp(f, _) => Pin::new(f).poll_write(cx, buf),
			RwFile::Lua(f) => Pin::new(f).poll_write(cx, buf),
		}
	}

	#[inline]
	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), io::Error>> {
		match &mut *self {
			RwFile::Tokio(f, _) => Pin::new(f).poll_flush(cx),
			RwFile::Sftp(f, _) => Pin::new(f).poll_flush(cx),
			RwFile::Lua(f) => Pin::new(f).poll_flush(cx),
		}
	}

	#[inline]
	fn poll_shutdown(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), io::Error>> {
		match &mut *self {
			RwFile::Tokio(f, _) => Pin::new(f).poll_shutdown(cx),
			RwFile::Sftp(f, _) => Pin::new(f).poll_shutdown(cx),
			RwFile::Lua(f) => Pin::new(f).poll_shutdown(cx),
		}
	}

	#[inline]
	fn poll_write_vectored(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		bufs: &[io::IoSlice<'_>],
	) -> std::task::Poll<Result<usize, io::Error>> {
		match &mut *self {
			RwFile::Tokio(f, _) => Pin::new(f).poll_write_vectored(cx, bufs),
			RwFile::Sftp(f, _) => Pin::new(f).poll_write_vectored(cx, bufs),
			RwFile::Lua(f) => Pin::new(f).poll_write_vectored(cx, bufs),
		}
	}

	#[inline]
	fn is_write_vectored(&self) -> bool {
		match self {
			RwFile::Tokio(f, _) => f.is_write_vectored(),
			RwFile::Sftp(f, _) => f.is_write_vectored(),
			RwFile::Lua(f) => f.is_write_vectored(),
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
