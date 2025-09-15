use std::{pin::Pin, task::Poll};

use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite};

pub struct RwFile(tokio::fs::File);

impl From<tokio::fs::File> for RwFile {
	fn from(value: tokio::fs::File) -> Self { Self(value) }
}

impl From<RwFile> for crate::provider::RwFile {
	fn from(value: RwFile) -> Self { Self::Local(value) }
}

impl AsyncRead for RwFile {
	#[inline]
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &mut tokio::io::ReadBuf<'_>,
	) -> Poll<std::io::Result<()>> {
		Pin::new(&mut self.0).poll_read(cx, buf)
	}
}

impl AsyncSeek for RwFile {
	#[inline]
	fn start_seek(mut self: Pin<&mut Self>, position: std::io::SeekFrom) -> std::io::Result<()> {
		Pin::new(&mut self.0).start_seek(position)
	}

	#[inline]
	fn poll_complete(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> Poll<std::io::Result<u64>> {
		Pin::new(&mut self.0).poll_complete(cx)
	}
}

impl AsyncWrite for RwFile {
	#[inline]
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &[u8],
	) -> Poll<Result<usize, std::io::Error>> {
		Pin::new(&mut self.0).poll_write(cx, buf)
	}

	#[inline]
	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> Poll<Result<(), std::io::Error>> {
		Pin::new(&mut self.0).poll_flush(cx)
	}

	#[inline]
	fn poll_shutdown(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> Poll<Result<(), std::io::Error>> {
		Pin::new(&mut self.0).poll_shutdown(cx)
	}

	#[inline]
	fn poll_write_vectored(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		bufs: &[std::io::IoSlice<'_>],
	) -> Poll<Result<usize, std::io::Error>> {
		Pin::new(&mut self.0).poll_write_vectored(cx, bufs)
	}

	#[inline]
	fn is_write_vectored(&self) -> bool { self.0.is_write_vectored() }
}
