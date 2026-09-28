use std::{fs::Permissions, io, path::{Path, PathBuf}, pin::Pin};

use tokio::{fs, io::{AsyncRead, AsyncSeek, AsyncWrite, ReadBuf}};
use yazi_shared::url::{AsUrl, Url, UrlLike};

use super::Attrs;
use crate::{file::File, stat::Stat};

pub struct RwFile {
	inner:    fs::File,
	pub path: PathBuf,
}

impl AsUrl for RwFile {
	fn as_url(&self) -> Url<'_> { self.path.as_url() }
}

impl UrlLike for RwFile {}

impl RwFile {
	pub(crate) fn new(inner: fs::File, path: impl Into<PathBuf>) -> Self {
		Self { inner, path: path.into() }
	}

	pub async fn create<P>(path: P) -> io::Result<Self>
	where
		P: AsRef<Path>,
	{
		let p = path.as_ref();

		Ok(Self::new(fs::File::create(p).await?, p))
	}

	pub async fn create_new<P>(path: P) -> io::Result<Self>
	where
		P: AsRef<Path>,
	{
		let p = path.as_ref();

		Ok(Self::new(fs::File::create_new(p).await?, p))
	}

	pub async fn into_file(self) -> io::Result<File> {
		let stat = self.metadata().await?;

		Ok(File { url: self.path.into(), stat, extra: Default::default() })
	}

	pub async fn metadata(&self) -> io::Result<Stat> {
		Ok(Stat::new(self.name().unwrap_or_default(), self.inner.metadata().await?))
	}

	pub async fn permissions(&self) -> io::Result<Permissions> {
		self.inner.metadata().await.map(|meta| meta.permissions())
	}

	pub async fn set_len(&self, size: u64) -> io::Result<()> { self.inner.set_len(size).await }

	pub async fn set_permissions(&self, permissions: Permissions) -> io::Result<()> {
		self.inner.set_permissions(permissions).await
	}

	pub async fn set_attrs(&self, attrs: Attrs) -> io::Result<()> {
		let (perm, times) = (attrs.try_into(), attrs.try_into());
		if perm.is_err() && times.is_err() {
			return Ok(());
		}

		let std = self.inner.try_clone().await?.into_std().await;
		tokio::task::spawn_blocking(move || {
			perm.map(|p| std.set_permissions(p)).ok();
			times.map(|t| std.set_times(t)).ok();
		})
		.await?;

		Ok(())
	}
}

impl AsyncRead for RwFile {
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &mut ReadBuf<'_>,
	) -> std::task::Poll<io::Result<()>> {
		Pin::new(&mut self.inner).poll_read(cx, buf)
	}
}

impl AsyncSeek for RwFile {
	fn start_seek(mut self: Pin<&mut Self>, position: io::SeekFrom) -> io::Result<()> {
		Pin::new(&mut self.inner).start_seek(position)
	}

	fn poll_complete(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<io::Result<u64>> {
		Pin::new(&mut self.inner).poll_complete(cx)
	}
}

impl AsyncWrite for RwFile {
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &[u8],
	) -> std::task::Poll<io::Result<usize>> {
		Pin::new(&mut self.inner).poll_write(cx, buf)
	}

	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<io::Result<()>> {
		Pin::new(&mut self.inner).poll_flush(cx)
	}

	fn poll_shutdown(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<io::Result<()>> {
		Pin::new(&mut self.inner).poll_shutdown(cx)
	}

	fn poll_write_vectored(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		bufs: &[io::IoSlice<'_>],
	) -> std::task::Poll<io::Result<usize>> {
		Pin::new(&mut self.inner).poll_write_vectored(cx, bufs)
	}

	fn is_write_vectored(&self) -> bool { self.inner.is_write_vectored() }
}
