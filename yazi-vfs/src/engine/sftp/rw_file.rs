use std::{io, pin::Pin};

use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite, ReadBuf};
use yazi_fs::{engine::Attrs, file::File};
use yazi_shared::url::{AsUrl, Url, UrlBuf, UrlLike};

use super::Stat;

pub struct RwFile {
	inner: yazi_sftp::fs::File,
	url:   UrlBuf,
}

impl AsUrl for RwFile {
	fn as_url(&self) -> Url<'_> { self.url.as_url() }
}

impl UrlLike for RwFile {}

impl RwFile {
	pub(crate) fn new(inner: yazi_sftp::fs::File, url: impl Into<UrlBuf>) -> Self {
		Self { inner, url: url.into() }
	}

	pub(crate) async fn metadata(&self) -> io::Result<yazi_fs::stat::Stat> {
		let name = self.name().unwrap_or_default().encoded_bytes();

		Ok(Stat::try_from((name, &self.inner.fstat().await?))?.0)
	}

	pub(crate) async fn into_file(self) -> io::Result<File> {
		let stat = self.metadata().await?;

		Ok(File { url: self.url, stat, extra: Default::default() })
	}

	pub(crate) async fn set_attrs(&self, attrs: Attrs) -> io::Result<()> {
		if let Ok(attrs) = super::Attrs(attrs).try_into() {
			self.inner.fsetstat(&attrs).await?;
		}

		Ok(())
	}

	pub(crate) async fn set_len(&self, size: u64) -> io::Result<()> {
		Ok(self.inner.fsetstat(&yazi_sftp::fs::Attrs { size: Some(size), ..Default::default() }).await?)
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
