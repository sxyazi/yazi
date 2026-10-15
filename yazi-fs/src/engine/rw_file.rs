use std::{io, ops::Deref, path::{Path, PathBuf}, pin::Pin};

use tokio::{fs, io::{AsyncRead, AsyncSeek, AsyncWrite, ReadBuf}};
use yazi_shared::url::{AsUrl, Url, UrlLike};

use super::FileHandle;
use crate::{file::File, stat::Stat};

pub struct RwFile {
	handle:   FileHandle,
	pub path: PathBuf,
}

impl Deref for RwFile {
	type Target = FileHandle;

	fn deref(&self) -> &Self::Target { &self.handle }
}

impl AsUrl for RwFile {
	fn as_url(&self) -> Url<'_> { self.path.as_url() }
}

impl UrlLike for RwFile {}

impl RwFile {
	pub(crate) fn new(tokio: fs::File, path: impl Into<PathBuf>) -> Self {
		Self { handle: FileHandle::new(tokio), path: path.into() }
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
		Ok(Stat::new(self.name().unwrap_or_default(), self.handle.metadata().await?))
	}
}

impl AsyncRead for RwFile {
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &mut ReadBuf<'_>,
	) -> std::task::Poll<io::Result<()>> {
		Pin::new(&mut self.handle.tokio).poll_read(cx, buf)
	}
}

impl AsyncSeek for RwFile {
	fn start_seek(mut self: Pin<&mut Self>, position: io::SeekFrom) -> io::Result<()> {
		Pin::new(&mut self.handle.tokio).start_seek(position)
	}

	fn poll_complete(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<io::Result<u64>> {
		Pin::new(&mut self.handle.tokio).poll_complete(cx)
	}
}

impl AsyncWrite for RwFile {
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &[u8],
	) -> std::task::Poll<io::Result<usize>> {
		Pin::new(&mut self.handle.tokio).poll_write(cx, buf)
	}

	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<io::Result<()>> {
		Pin::new(&mut self.handle.tokio).poll_flush(cx)
	}

	fn poll_shutdown(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<io::Result<()>> {
		Pin::new(&mut self.handle.tokio).poll_shutdown(cx)
	}

	fn poll_write_vectored(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		bufs: &[io::IoSlice<'_>],
	) -> std::task::Poll<io::Result<usize>> {
		Pin::new(&mut self.handle.tokio).poll_write_vectored(cx, bufs)
	}

	fn is_write_vectored(&self) -> bool { self.handle.is_write_vectored() }
}
