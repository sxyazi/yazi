use std::{io, pin::Pin, task::{Context, Poll}};

use tokio::io::{AsyncRead, AsyncWrite, BufReader, BufWriter, ReadBuf};

use super::RwFile;

pub(super) enum FileCursor {
	Local(yazi_fs::engine::FileCursor),
	Reader(BufReader<RwFile>),
	Writer(BufWriter<RwFile>),
}

impl FileCursor {
	pub(super) async fn reader(file: &RwFile, offset: u64) -> io::Result<Self> {
		let file = match file {
			RwFile::Local(f) => return Ok(Self::Local(f.cursor(offset).await?)),
			RwFile::Sftp(f) => f.cursor(offset).into(),
			RwFile::Lua(f) => f.cursor(offset).into(),
		};

		Ok(Self::Reader(BufReader::with_capacity(512 * 1024, file)))
	}

	pub(super) async fn writer(file: &RwFile, offset: u64) -> io::Result<Self> {
		let file = match file {
			RwFile::Local(f) => return Ok(Self::Local(f.cursor(offset).await?)),
			RwFile::Sftp(f) => f.cursor(offset).into(),
			RwFile::Lua(f) => f.cursor(offset).into(),
		};

		Ok(Self::Writer(BufWriter::with_capacity(512 * 1024, file)))
	}

	pub(super) fn get_mut(&mut self) -> &mut (dyn AsyncWrite + Unpin + Send) {
		match self {
			Self::Local(file) => file,
			Self::Reader(file) => file.get_mut(),
			Self::Writer(file) => file.get_mut(),
		}
	}
}

impl AsyncRead for FileCursor {
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &mut ReadBuf<'_>,
	) -> Poll<io::Result<()>> {
		poll_rw!(self, poll_read(cx, buf); Local, Reader, Writer)
	}
}

impl AsyncWrite for FileCursor {
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &[u8],
	) -> Poll<io::Result<usize>> {
		poll_rw!(self, poll_write(cx, buf); Local, Reader, Writer)
	}

	fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		poll_rw!(self, poll_flush(cx); Local, Reader, Writer)
	}

	fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		poll_rw!(self, poll_shutdown(cx); Local, Reader, Writer)
	}
}
