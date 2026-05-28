use std::{fmt::{self, Debug}, io::{self, BufWriter}, ops::Deref};

use parking_lot::Mutex;

use crate::Handle;

#[derive(Clone, Copy)]
pub struct TtyWriter<'a>(pub(super) &'a Mutex<BufWriter<Handle>>);

impl Deref for TtyWriter<'_> {
	type Target = Mutex<BufWriter<Handle>>;

	fn deref(&self) -> &Self::Target { self.0 }
}

impl Debug for TtyWriter<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("TtyWriter") }
}

impl io::Write for TtyWriter<'_> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> { self.0.lock().write(buf) }

	fn flush(&mut self) -> io::Result<()> { self.0.lock().flush() }
}

#[cfg(unix)]
impl std::os::fd::AsFd for TtyWriter<'_> {
	fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
		use std::os::fd::{AsRawFd, BorrowedFd};

		let raw = self.lock().get_ref().as_raw_fd();
		unsafe { BorrowedFd::borrow_raw(raw) }
	}
}

#[cfg(windows)]
impl std::os::windows::io::AsRawHandle for TtyWriter<'_> {
	fn as_raw_handle(&self) -> std::os::windows::io::RawHandle {
		self.lock().get_ref().as_raw_handle()
	}
}
