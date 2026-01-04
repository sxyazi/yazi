use std::{io::{self, SeekFrom}, pin::Pin, sync::Arc, task::{Context, Poll, ready}, time::Duration};

use tokio::{io::{AsyncRead, AsyncSeek, AsyncWrite, ReadBuf}, time::{Timeout, timeout}};

use crate::{Error, Operator, Packet, Receiver, Session, fs::Attrs, requests};

pub struct File {
	session: Arc<Session>,

	handle: String,
	cursor: u64,
	closed: bool,

	close_rx: Option<Timeout<Receiver>>,
	read_rx:  Option<Receiver>,
	seek_rx:  Option<SeekState>,
	write_rx: Option<(Receiver, usize)>,
	flush_rx: Option<Timeout<Receiver>>,
}

enum SeekState {
	NonBlocking(u64),
	Blocking(i64, Timeout<Receiver>),
}

impl Unpin for File {}

impl Drop for File {
	fn drop(&mut self) {
		if !self.closed {
			Operator::from(&self.session).close(&self.handle).ok();
		}
	}
}

impl File {
	pub(crate) fn new(session: &Arc<Session>, handle: impl Into<String>) -> Self {
		Self {
			session: session.clone(),

			handle: handle.into(),
			closed: false,
			cursor: 0,

			close_rx: None,
			read_rx:  None,
			seek_rx:  None,
			write_rx: None,
			flush_rx: None,
		}
	}

	pub async fn fstat(&self) -> Result<Attrs, Error> {
		Operator::from(&self.session).fstat(&self.handle).await
	}

	pub async fn fsetstat(&self, attrs: &Attrs) -> Result<(), Error> {
		Operator::from(&self.session).fsetstat(&self.handle, attrs).await
	}
}

impl AsyncRead for File {
	fn poll_read(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &mut ReadBuf<'_>,
	) -> Poll<io::Result<()>> {
		let me = unsafe { self.get_unchecked_mut() };

		if me.read_rx.is_none() {
			let max = buf.remaining().min(261120) as u32;
			me.read_rx = Some(Operator::from(&me.session).read(&me.handle, me.cursor, max)?);
		}

		let result = ready!(Pin::new(me.read_rx.as_mut().unwrap()).poll(cx));
		me.read_rx = None;

		Poll::Ready(match result {
			Ok(Packet::Data(data)) => {
				let len = buf.remaining().min(data.data.len());
				me.cursor += len as u64;
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
	fn start_seek(mut self: Pin<&mut Self>, position: io::SeekFrom) -> io::Result<()> {
		if self.seek_rx.is_some() {
			return Err(io::Error::other(
				"other file operation is pending, call poll_complete before start_seek",
			));
		}

		self.seek_rx = Some(match position {
			SeekFrom::Start(n) => SeekState::NonBlocking(n),
			SeekFrom::Current(n) => self
				.cursor
				.checked_add_signed(n)
				.map(SeekState::NonBlocking)
				.ok_or_else(|| io::Error::other("seeking to a negative or overflowed position"))?,
			SeekFrom::End(n) => SeekState::Blocking(
				n,
				timeout(
					Duration::from_secs(45),
					self.session.send_sync(requests::Fstat::new(&self.handle))?,
				),
			),
		});

		Ok(())
	}

	fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
		let me = unsafe { self.get_unchecked_mut() };

		let Some(state) = &mut me.seek_rx else {
			return Poll::Ready(Ok(me.cursor));
		};

		fn imp(cx: &mut Context<'_>, state: &mut SeekState) -> Poll<io::Result<u64>> {
			use Poll::Ready;
			let (n, rx) = match state {
				SeekState::NonBlocking(n) => return Ready(Ok(*n)),
				SeekState::Blocking(n, rx) => (n, rx),
			};

			let Ok(result) = ready!(unsafe { Pin::new_unchecked(rx) }.poll(cx)) else {
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

		let result = ready!(imp(cx, state));
		if let Ok(n) = result {
			me.cursor = n;
		}

		me.seek_rx = None;
		Poll::Ready(result)
	}
}

impl AsyncWrite for File {
	fn poll_write(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &[u8],
	) -> Poll<Result<usize, io::Error>> {
		let me = unsafe { self.get_unchecked_mut() };

		let (rx, len) = match &mut me.write_rx {
			Some((rx, len)) => (rx, *len),
			None => {
				let max = buf.len().min(261120);
				let rx = Operator::from(&me.session).write(&me.handle, me.cursor, &buf[..max])?;
				(&mut me.write_rx.get_or_insert((rx, max)).0, max)
			}
		};

		let result = ready!(Pin::new(rx).poll(cx));
		me.write_rx = None;

		Poll::Ready(match result {
			Ok(Packet::Status(status)) if status.is_ok() => {
				me.cursor += len as u64;
				Ok(len)
			}
			Ok(Packet::Status(status)) => Err(Error::Status(status).into()),
			Ok(_) => Err(Error::Packet("not a Status").into()),
			Err(e) => Err(Error::from(e).into()),
		})
	}

	fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
		let me = unsafe { self.get_unchecked_mut() };

		if me.flush_rx.is_none() {
			match Operator::from(&me.session).fsync(&me.handle) {
				Ok(rx) => me.flush_rx = Some(timeout(Duration::from_secs(45), rx)),
				Err(Error::Unsupported) => return Poll::Ready(Ok(())),
				Err(e) => Err(e)?,
			}
		}

		let rx = unsafe { Pin::new_unchecked(me.flush_rx.as_mut().unwrap()) };
		let result = ready!(rx.poll(cx));
		me.flush_rx = None;

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
		let me = unsafe { self.get_unchecked_mut() };

		if me.close_rx.is_none() {
			me.close_rx =
				Some(timeout(Duration::from_secs(10), Operator::from(&me.session).close(&me.handle)?));
		}

		let rx = unsafe { Pin::new_unchecked(me.close_rx.as_mut().unwrap()) };
		let result = ready!(rx.poll(cx));
		me.close_rx = None;

		let Ok(result) = result else {
			return Poll::Ready(Err(Error::Timeout.into()));
		};

		Poll::Ready(match result {
			Ok(Packet::Status(status)) if status.is_ok() => {
				me.closed = true;
				Ok(())
			}
			Ok(Packet::Status(status)) => Err(Error::Status(status).into()),
			Ok(_) => Err(Error::Packet("not a Status").into()),
			Err(e) => Err(Error::from(e).into()),
		})
	}
}
