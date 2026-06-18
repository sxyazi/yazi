use std::{io::{self, Read}, os::{fd::{AsFd, BorrowedFd}, unix::net::UnixStream}, time::Duration};

use parking_lot::Mutex;
use rustix::{event::Timespec, termios};
use signal_hook::consts::SIGWINCH;
use yazi_tty::{TtyReader, TtyWriter};

use crate::{Timeout, event::Event, parser::Parser, waker::Waker};

#[derive(Debug)]
pub struct EventSource<'a> {
	pub(super) parser: Mutex<Parser>,
	reader:            TtyReader<'a>,
	writer:            TtyWriter<'a>,
	waker:             Waker,

	sigwinch_id:   signal_hook::SigId,
	sigwinch_read: UnixStream,
}

impl Drop for EventSource<'_> {
	fn drop(&mut self) { signal_hook::low_level::unregister(self.sigwinch_id); }
}

impl<'a> EventSource<'a> {
	pub(crate) fn new(reader: TtyReader<'a>, writer: TtyWriter<'a>) -> io::Result<Self> {
		let (sigwinch_read, sigwinch_write) = UnixStream::pair()?;
		let sigwinch_id = signal_hook::low_level::pipe::register(SIGWINCH, sigwinch_write)?;
		sigwinch_read.set_nonblocking(true)?;

		Ok(Self {
			parser: Mutex::new(Parser::default()),
			reader,
			writer,
			waker: Waker::new()?,

			sigwinch_id,
			sigwinch_read,
		})
	}

	pub fn wake(&self) -> io::Result<()> { self.waker.wake() }

	pub(crate) fn try_fill(&self, timeout: Timeout) -> io::Result<()> {
		let mut reader = self.reader.lock();
		let [read_ready, sigwinch_ready, wakeup_ready] =
			poll([reader.as_fd(), self.sigwinch_read.as_fd(), self.waker.as_fd()], timeout.leftover())?;

		// Stop waiting for events.
		if wakeup_ready {
			while read_complete(&*self.waker, &mut [0u8; 1024])? != 0 {}
			return Err(io::Error::from(io::ErrorKind::ConnectionAborted));
		}

		// More input is ready.
		if read_ready {
			let mut buf = [0u8; 1024];

			let len = read_complete(&mut *reader, &mut buf)?;
			if len == 0 {
				return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
			}

			let mut parser = self.parser.lock();
			parser.parse(&buf[..len]);

			if len < buf.len() {
				parser.flush();
			}
			return Ok(());
		}

		// SIGWINCH signal received, indicating a terminal resize.
		if sigwinch_ready {
			while read_complete(&self.sigwinch_read, &mut [0; 1024])? != 0 {}
			self.parser.lock().emit(Event::Resize(termios::tcgetwinsize(self.writer)?.into()));
		}

		Ok(())
	}
}

fn read_complete<F: Read>(mut reader: F, buf: &mut [u8]) -> io::Result<usize> {
	loop {
		match reader.read(buf) {
			Ok(len) => return Ok(len),
			Err(e) => match e.kind() {
				io::ErrorKind::WouldBlock => return Ok(0),
				io::ErrorKind::Interrupted => continue,
				_ => return Err(e),
			},
		}
	}
}

/// A small abstraction over platform specific polling behavior.
///
/// macOS `poll(2)` doesn't work on file descriptors to `/dev/tty` so we need to
/// use `select(2)` instead. This provides a function which abstracts over the
/// parts of `poll(2)` and `select(2)` we want. Specifically we are looking for
/// `POLLIN` events from `poll(2)` and we consider that to be "ready."
///
/// This module is not meant to be generic. We consider `POLLIN` to be "ready"
/// and do not look at other poll flags. For the sake of simplicity we also only
/// allow polling exactly three FDs at a time - the exact amount we need for the
/// event source.
fn poll(fds: [BorrowedFd<'_>; 3], timeout: Option<Duration>) -> io::Result<[bool; 3]> {
	#[cfg(not(target_os = "macos"))]
	fn poll3(fds: [BorrowedFd<'_>; 3], timeout: Option<&Timespec>) -> io::Result<[bool; 3]> {
		use rustix::event::{PollFd, PollFlags};
		let mut fds = [
			PollFd::new(&fds[0], PollFlags::IN),
			PollFd::new(&fds[1], PollFlags::IN),
			PollFd::new(&fds[2], PollFlags::IN),
		];

		rustix::event::poll(&mut fds, timeout)?;

		Ok([
			fds[0].revents().contains(PollFlags::IN),
			fds[1].revents().contains(PollFlags::IN),
			fds[2].revents().contains(PollFlags::IN),
		])
	}

	#[cfg(target_os = "macos")]
	fn select3(fds: [BorrowedFd<'_>; 3], timeout: Option<&Timespec>) -> io::Result<[bool; 3]> {
		use std::os::fd::AsRawFd;

		use rustix::event::{FdSetElement, FdSetIter, fd_set_insert, fd_set_num_elements};

		let fds = [fds[0].as_raw_fd(), fds[1].as_raw_fd(), fds[2].as_raw_fd()];
		let nfds = fds.iter().copied().max().unwrap() + 1;

		let mut set = vec![FdSetElement::default(); fd_set_num_elements(fds.len(), nfds)];
		for fd in fds {
			fd_set_insert(&mut set, fd);
		}
		unsafe { rustix::event::select(nfds, Some(&mut set), None, None, timeout) }?;

		let mut result = [false; 3];
		for (fd, ready) in fds.iter().copied().zip(result.iter_mut()) {
			if FdSetIter::new(&set).any(|x| x == fd) {
				*ready = true;
			}
		}
		Ok(result)
	}

	let timespec = timeout.map(|t| t.try_into()).transpose().map_err(|_| {
		io::Error::new(io::ErrorKind::InvalidInput, "timeout is too large for the platform")
	})?;

	#[cfg(not(target_os = "macos"))]
	return poll3(fds, timespec.as_ref());
	#[cfg(target_os = "macos")]
	return select3(fds, timespec.as_ref());
}
