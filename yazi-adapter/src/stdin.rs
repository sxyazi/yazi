use std::{ops::{Deref, DerefMut}, time::Duration};

pub struct AsyncStdin {
	inner: std::io::StdinLock<'static>,
	#[cfg(unix)]
	fds:   libc::fd_set,
}

impl Deref for AsyncStdin {
	type Target = std::io::StdinLock<'static>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for AsyncStdin {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

#[cfg(unix)]
impl AsyncStdin {
	pub fn new(inner: std::io::StdinLock<'static>) -> Self {
		let mut me = Self { inner, fds: unsafe { std::mem::MaybeUninit::zeroed().assume_init() } };
		me.reset();
		me
	}

	pub fn poll(&mut self, timeout: Duration) -> std::io::Result<bool> {
		use std::os::unix::io::AsRawFd;

		let mut tv = libc::timeval {
			tv_sec:  timeout.as_secs() as libc::time_t,
			tv_usec: timeout.subsec_micros() as libc::suseconds_t,
		};

		let result = unsafe {
			libc::select(
				self.inner.as_raw_fd() + 1,
				&mut self.fds,
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				&mut tv,
			)
		};

		match result {
			-1 => Err(std::io::Error::last_os_error()),
			0 => Ok(false),
			_ => {
				self.reset();
				Ok(true)
			}
		}
	}

	fn reset(&mut self) {
		use std::os::unix::io::AsRawFd;

		unsafe {
			libc::FD_ZERO(&mut self.fds);
			libc::FD_SET(self.inner.as_raw_fd(), &mut self.fds);
		}
	}
}

#[cfg(windows)]
impl AsyncStdin {
	pub fn new(inner: std::io::StdinLock<'static>) -> Self { Self { inner } }

	pub fn poll(&mut self, timeout: Duration) -> std::io::Result<bool> {
		use std::os::windows::io::AsRawHandle;

		use windows_sys::Win32::{Foundation::WAIT_TIMEOUT, System::Threading::WaitForSingleObject};

		let handle = self.inner.as_raw_handle();
		let millis = timeout.as_millis();
		match unsafe { WaitForSingleObject(handle, millis as u32) } {
			WAIT_TIMEOUT => Ok(false),
			_ => Ok(true),
		}
	}
}
