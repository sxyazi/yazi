use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncWrite};

pub enum RwFile {
	Tokio(tokio::fs::File),
	Sftp(Box<yazi_sftp::fs::File>),
}

impl From<tokio::fs::File> for RwFile {
	fn from(f: tokio::fs::File) -> Self { Self::Tokio(f) }
}

impl From<yazi_sftp::fs::File> for RwFile {
	fn from(f: yazi_sftp::fs::File) -> Self { Self::Sftp(Box::new(f)) }
}

impl AsyncRead for RwFile {
	#[inline]
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &mut tokio::io::ReadBuf<'_>,
	) -> std::task::Poll<std::io::Result<()>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_read(cx, buf),
			Self::Sftp(f) => Pin::new(f).poll_read(cx, buf),
		}
	}
}

impl AsyncWrite for RwFile {
	#[inline]
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &[u8],
	) -> std::task::Poll<Result<usize, std::io::Error>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_write(cx, buf),
			Self::Sftp(f) => Pin::new(f).poll_write(cx, buf),
		}
	}

	#[inline]
	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), std::io::Error>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_flush(cx),
			Self::Sftp(f) => Pin::new(f).poll_flush(cx),
		}
	}

	#[inline]
	fn poll_shutdown(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), std::io::Error>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_shutdown(cx),
			Self::Sftp(f) => Pin::new(f).poll_shutdown(cx),
		}
	}

	#[inline]
	fn poll_write_vectored(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		bufs: &[std::io::IoSlice<'_>],
	) -> std::task::Poll<Result<usize, std::io::Error>> {
		match &mut *self {
			Self::Tokio(f) => Pin::new(f).poll_write_vectored(cx, bufs),
			Self::Sftp(f) => Pin::new(f).poll_write_vectored(cx, bufs),
		}
	}

	#[inline]
	fn is_write_vectored(&self) -> bool {
		match self {
			Self::Tokio(f) => f.is_write_vectored(),
			Self::Sftp(f) => f.is_write_vectored(),
		}
	}
}
