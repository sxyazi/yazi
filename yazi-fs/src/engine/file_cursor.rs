#[cfg(unix)]
use std::os::unix::fs::FileExt;
#[cfg(windows)]
use std::os::windows::fs::FileExt;
use std::{fs::File, io, mem, pin::Pin, sync::Arc, task::{Context, Poll, ready}};

use tokio::{io::{AsyncRead, AsyncWrite, ReadBuf}, task::spawn_blocking};
use tokio_util::task::AbortOnDropHandle;

pub struct FileCursor {
	file:   Arc<File>,
	offset: u64,
	buf:    io::Cursor<Vec<u8>>,
	task:   Option<AbortOnDropHandle<Transfer>>,
}

impl FileCursor {
	pub fn new(file: Arc<File>, offset: u64) -> Self {
		Self { file, offset, buf: Default::default(), task: None }
	}

	fn take(&mut self) -> Self {
		Self {
			file:   self.file.clone(),
			offset: self.offset,
			buf:    mem::take(&mut self.buf),
			task:   None,
		}
	}

	fn read_at(&mut self) -> io::Result<usize> {
		let Self { file, offset, buf, .. } = self;

		loop {
			#[cfg(unix)]
			let result = file.read_at(buf.get_mut(), *offset);
			#[cfg(windows)]
			let result = file.seek_read(buf.get_mut(), *offset);

			match result {
				Ok(n) => {
					*offset += n as u64;
					return Ok(n);
				}
				Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
				Err(e) => return Err(e),
			}
		}
	}

	fn write_all_at(&mut self) -> io::Result<usize> {
		let Self { file, offset, buf, .. } = self;

		let mut written = 0;
		loop {
			#[cfg(unix)]
			let result = file.write_at(&buf.get_ref()[written..], *offset);
			#[cfg(windows)]
			let result = file.seek_write(&buf.get_ref()[written..], *offset);

			match result {
				Ok(0) if written < buf.get_ref().len() => return Err(io::ErrorKind::WriteZero.into()),
				Ok(n) => {
					*offset += n as u64;
					written += n;
					if written == buf.get_ref().len() {
						return Ok(written);
					}
				}
				Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
				Err(e) => return Err(e),
			}
		}
	}

	fn poll_transfer(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<Option<(usize, bool)>>> {
		let Some(task) = &mut self.task else {
			return Poll::Ready(Ok(None));
		};

		let result = ready!(Pin::new(task).poll(cx));
		self.task = None;

		let Transfer { buf, offset, result, writing } = result?;
		self.buf = io::Cursor::new(buf);
		self.offset = offset; // A write may fail after committing some bytes, preserve its actual offset even on error.

		Poll::Ready(result.map(|n| Some((n, writing))))
	}
}

impl AsyncRead for FileCursor {
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &mut ReadBuf<'_>,
	) -> Poll<io::Result<()>> {
		let completed = ready!(self.poll_transfer(cx))?;
		if buf.remaining() == 0 {
			return Poll::Ready(Ok(()));
		} else if completed == Some((0, false)) {
			return Poll::Ready(Ok(()));
		}

		if self.buf.position() == self.buf.get_ref().len() as u64 {
			self.buf.get_mut().resize(512 * 1024, 0);
			self.task = Some(Transfer::spawn(&mut self, false));
			ready!(self.poll_transfer(cx))?;
		}

		Pin::new(&mut self.buf).poll_read(cx, buf)
	}
}

impl AsyncWrite for FileCursor {
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &[u8],
	) -> Poll<io::Result<usize>> {
		ready!(self.poll_transfer(cx))?;

		// The offset includes unread buffered bytes, subtract them before writing.
		self.offset -= self.buf.get_ref().len() as u64 - self.buf.position();

		buf.clone_into(self.buf.get_mut());
		self.task = Some(Transfer::spawn(&mut self, true));

		// Report accepted bytes now; subsequent operations wait for the write and propagate its errors.
		Poll::Ready(Ok(buf.len()))
	}

	fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		ready!(self.poll_transfer(cx))?;
		Poll::Ready(Ok(()))
	}

	fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		self.poll_flush(cx)
	}
}

// --- Transfer
struct Transfer {
	buf:     Vec<u8>,
	offset:  u64,
	result:  io::Result<usize>,
	writing: bool,
}

impl Transfer {
	fn spawn(cursor: &mut FileCursor, writing: bool) -> AbortOnDropHandle<Self> {
		let mut cursor = cursor.take();

		// Keep the handle and buffer alive until the system call returns, even if the cursor is dropped.
		AbortOnDropHandle::new(spawn_blocking(move || {
			let result = if writing { cursor.write_all_at() } else { cursor.read_at() };

			cursor.buf.get_mut().truncate(if writing { 0 } else { *result.as_ref().unwrap_or(&0) });

			Self { buf: cursor.buf.into_inner(), offset: cursor.offset, result, writing }
		}))
	}
}
