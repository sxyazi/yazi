use std::{fmt::Debug, io::Read, ops::Deref};

use parking_lot::Mutex;

use crate::Handle;

#[derive(Clone, Copy)]
pub struct TtyReader<'a>(pub(super) &'a Mutex<Handle>);

impl Deref for TtyReader<'_> {
	type Target = Mutex<Handle>;

	fn deref(&self) -> &Self::Target { self.0 }
}

impl Debug for TtyReader<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("TtyReader") }
}

impl Read for TtyReader<'_> {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> { self.0.lock().read(buf) }
}

#[cfg(unix)]
impl std::os::fd::AsFd for TtyReader<'_> {
	fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
		use std::os::fd::{AsRawFd, BorrowedFd};

		let raw = self.lock().as_raw_fd();
		unsafe { BorrowedFd::borrow_raw(raw) }
	}
}

#[cfg(windows)]
impl std::os::windows::io::AsRawHandle for TtyReader<'_> {
	fn as_raw_handle(&self) -> std::os::windows::io::RawHandle { self.lock().as_raw_handle() }
}
