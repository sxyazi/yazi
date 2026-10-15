use std::{ffi::CStr, io, os::fd::{AsFd, AsRawFd, BorrowedFd}};

use yazi_shim::nonneg_ok;

/// macOS extended attributes accessed through a borrowed file descriptor.
pub struct Xattr<'a> {
	fd:      BorrowedFd<'a>,
	options: i32,
}

impl<'a> Xattr<'a> {
	pub fn new(file: &'a impl AsFd, options: i32) -> Self { Self { fd: file.as_fd(), options } }

	pub fn get(&self, name: &CStr) -> io::Result<Vec<u8>> {
		let mut data = Vec::new();
		for _ in 0..4 {
			data.resize(self.read_at(name, &mut [], 0)?, 0);

			match self.read_at(name, &mut data, 0) {
				Ok(size) => {
					data.truncate(size);
					return Ok(data);
				}
				// An attribute can grow between sizing and reading; repeat with a larger buffer, as copyfile does.
				Err(e) if e.raw_os_error() == Some(libc::ERANGE) => {}
				Err(e) => return Err(e),
			}
		}

		Err(io::Error::from_raw_os_error(libc::ERANGE))
	}

	pub fn set(&self, name: &CStr, data: &[u8]) -> io::Result<()> { self.write_at(name, data, 0) }

	pub fn remove(&self, name: &CStr) -> io::Result<()> {
		nonneg_ok(unsafe { libc::fremovexattr(self.fd.as_raw_fd(), name.as_ptr(), self.options) })
	}

	/// Nonzero positions are supported only for com.apple.ResourceFork.
	pub fn read_at(&self, name: &CStr, buf: &mut [u8], pos: u32) -> io::Result<usize> {
		let size = unsafe {
			libc::fgetxattr(
				self.fd.as_raw_fd(),
				name.as_ptr(),
				buf.as_mut_ptr().cast(),
				buf.len(),
				pos,
				self.options,
			)
		};

		usize::try_from(size).map_err(|_| io::Error::last_os_error())
	}

	/// Nonzero positions are supported only for com.apple.ResourceFork.
	pub fn write_at(&self, name: &CStr, data: &[u8], pos: u32) -> io::Result<()> {
		nonneg_ok(unsafe {
			libc::fsetxattr(
				self.fd.as_raw_fd(),
				name.as_ptr(),
				data.as_ptr().cast(),
				data.len(),
				pos,
				self.options,
			)
		})
	}
}
