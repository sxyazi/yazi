use std::{io::{self, SeekFrom}, ops::Deref, pin::Pin, sync::Arc, task::{Context, Poll, ready}, time::Duration};

use pin_project::pin_project;
use tokio::{io::{AsyncRead, AsyncSeek, AsyncWrite, ReadBuf}, time::{Timeout, timeout}};

use super::FileHandle;
use crate::{Error, Packet, Receiver, Session};

#[pin_project]
pub struct File {
	handle: Arc<FileHandle>,
	cursor: u64,

	read_rx:  Option<Receiver>,
	#[pin]
	seek_rx:  Option<SeekState>,
	write_rx: Option<(Receiver, usize)>,
	#[pin]
	flush_rx: Option<Timeout<Receiver>>,
}

impl Deref for File {
	type Target = FileHandle;

	fn deref(&self) -> &Self::Target { &self.handle }
}

impl File {
	pub(crate) fn new(session: &Arc<Session>, handle: impl Into<String>) -> Self {
		Self {
			handle: Arc::new(FileHandle::new(session, handle)),
			cursor: 0,

			read_rx:  None,
			seek_rx:  None,
			write_rx: None,
			flush_rx: None,
		}
	}

	pub fn cursor(&self, offset: u64) -> Self {
		Self {
			handle: self.handle.clone(),
			cursor: offset,

			read_rx:  None,
			seek_rx:  None,
			write_rx: None,
			flush_rx: None,
		}
	}
}

impl AsyncRead for File {
	fn poll_read(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &mut ReadBuf<'_>,
	) -> Poll<io::Result<()>> {
		let me = self.project();

		if me.read_rx.is_none() {
			let max = buf.remaining().min(261120) as u32;
			*me.read_rx = Some(me.handle.read(*me.cursor, max)?);
		}

		let result = ready!(Pin::new(me.read_rx.as_mut().unwrap()).poll(cx));
		*me.read_rx = None;

		Poll::Ready(match result {
			Ok(Packet::Data(data)) => {
				let len = buf.remaining().min(data.data.len());
				*me.cursor += len as u64;
				buf.put_slice(&data.data[..len]);
				Ok(())
			}
			Ok(Packet::Status(status)) if status.is_eof() => Ok(()),
			Ok(Packet::Status(status)) => Err(Error::Status(status).into()),
			Ok(_) => Err(Error::Packet("not a Data or Status").into()),
			Err(e) => Err(Error::from(e).into()),
		})
	}
}

impl AsyncSeek for File {
	fn start_seek(self: Pin<&mut Self>, position: io::SeekFrom) -> io::Result<()> {
		let mut me = self.project();

		if me.seek_rx.is_some() {
			return Err(io::Error::other(
				"other file operation is pending, call poll_complete before start_seek",
			));
		}

		me.seek_rx.set(Some(match position {
			SeekFrom::Start(n) => SeekState::NonBlocking(n),
			SeekFrom::Current(n) => me
				.cursor
				.checked_add_signed(n)
				.map(SeekState::NonBlocking)
				.ok_or_else(|| io::Error::other("seeking to a negative or overflowed position"))?,
			SeekFrom::End(n) => {
				SeekState::Blocking(n, timeout(Duration::from_secs(45), me.handle.fstat_sync()?))
			}
		}));

		Ok(())
	}

	fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
		let mut me = self.project();

		let Some(state) = me.seek_rx.as_mut().as_pin_mut() else {
			return Poll::Ready(Ok(*me.cursor));
		};

		let result = ready!(state.poll(cx));
		if let Ok(n) = result {
			*me.cursor = n;
		}

		me.seek_rx.set(None);
		Poll::Ready(result)
	}
}

impl AsyncWrite for File {
	fn poll_write(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &[u8],
	) -> Poll<Result<usize, io::Error>> {
		let me = self.project();

		let (rx, len) = match &mut *me.write_rx {
			Some((rx, len)) => (rx, *len),
			None => {
				let max = buf.len().min(261120);
				let rx = me.handle.write(*me.cursor, &buf[..max])?;
				(&mut me.write_rx.get_or_insert((rx, max)).0, max)
			}
		};

		let result = ready!(Pin::new(rx).poll(cx));
		*me.write_rx = None;

		Poll::Ready(match result {
			Ok(Packet::Status(status)) if status.is_ok() => {
				*me.cursor += len as u64;
				Ok(len)
			}
			Ok(Packet::Status(status)) => Err(Error::Status(status).into()),
			Ok(_) => Err(Error::Packet("not a Status").into()),
			Err(e) => Err(Error::from(e).into()),
		})
	}

	fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
		if self.write_rx.is_some() {
			ready!(self.as_mut().poll_write(cx, &[]))?;
		}

		let mut me = self.project();
		if me.flush_rx.is_none() {
			match me.handle.fsync() {
				Ok(rx) => me.flush_rx.set(Some(timeout(Duration::from_secs(45), rx))),
				Err(Error::Unsupported) => return Poll::Ready(Ok(())),
				Err(e) => Err(e)?,
			}
		}

		let rx = me.flush_rx.as_mut().as_pin_mut().unwrap();
		let result = ready!(rx.poll(cx));
		me.flush_rx.set(None);

		let Ok(result) = result else {
			return Poll::Ready(Err(Error::Timeout.into()));
		};

		Poll::Ready(match result {
			Ok(Packet::Status(status)) if status.is_ok() => Ok(()),
			Ok(Packet::Status(status)) => Err(Error::Status(status).into()),
			Ok(_) => Err(Error::Packet("not a Status").into()),
			Err(e) => Err(Error::from(e).into()),
		})
	}

	fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
		self.poll_flush(cx)
	}
}

// --- SeekState
#[pin_project(project = SeekStateProj)]
enum SeekState {
	NonBlocking(u64),
	Blocking(i64, #[pin] Timeout<Receiver>),
}

impl SeekState {
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
		use Poll::Ready;

		let (n, rx) = match self.project() {
			SeekStateProj::NonBlocking(n) => return Ready(Ok(*n)),
			SeekStateProj::Blocking(n, rx) => (n, rx),
		};

		let Ok(result) = ready!(rx.poll(cx)) else {
			return Ready(Err(Error::Timeout.into()));
		};

		let packet = match result {
			Ok(Packet::Attrs(packet)) => packet,
			Ok(_) => return Ready(Err(Error::Packet("not an Attrs").into())),
			Err(e) => return Ready(Err(Error::from(e).into())),
		};

		let Some(size) = packet.attrs.size else {
			return Ready(Err(io::Error::other("could not get file size for seeking from end")));
		};

		Ready(
			size
				.checked_add_signed(*n)
				.ok_or_else(|| io::Error::other("seeking to a negative or overflowed position")),
		)
	}
}
