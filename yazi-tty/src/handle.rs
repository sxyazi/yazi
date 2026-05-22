use std::{io::{Error, Read, Write}, ptr, time::Duration};

use tracing::error;

#[derive(Debug)]
pub struct Handle {
	#[cfg(unix)]
	inner: std::os::fd::RawFd,
	#[cfg(windows)]
	inner: std::os::windows::io::RawHandle,
	close: bool,
}

// Windows HANDLEs are kernel object references and it is safe to use them in
// any threads.
#[cfg(windows)]
unsafe impl Send for Handle {}

impl Drop for Handle {
	fn drop(&mut self) {
		#[cfg(unix)]
		if self.close {
			unsafe { libc::close(self.inner) };
		}
		#[cfg(windows)]
		if self.close {
			unsafe { windows_sys::Win32::Foundation::CloseHandle(self.inner) };
		}
	}
}

#[cfg(unix)]
impl From<Handle> for std::process::Stdio {
	fn from(value: Handle) -> Self {
		use std::os::fd::{FromRawFd, IntoRawFd, OwnedFd};

		Self::from(unsafe { OwnedFd::from_raw_fd(value.into_raw_fd()) })
	}
}

#[cfg(windows)]
impl From<Handle> for std::process::Stdio {
	fn from(value: Handle) -> Self {
		use std::os::windows::io::{FromRawHandle, IntoRawHandle, OwnedHandle};

		Self::from(unsafe { OwnedHandle::from_raw_handle(value.into_raw_handle()) })
	}
}

impl Read for Handle {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		#[cfg(unix)]
		{
			match unsafe { libc::read(self.inner, buf.as_mut_ptr() as *mut _, buf.len()) } {
				-1 => Err(Error::last_os_error()),
				n => Ok(n as usize),
			}
		}
		#[cfg(windows)]
		{
			use windows_sys::Win32::Storage::FileSystem::ReadFile;
			use yazi_shim::bool_ok;

			let mut len = 0;
			bool_ok(unsafe {
				ReadFile(self.inner, buf.as_mut_ptr(), buf.len() as u32, &mut len, ptr::null_mut())
			})?;
			Ok(len as usize)
		}
	}
}

impl Write for Handle {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		#[cfg(unix)]
		{
			match unsafe { libc::write(self.inner, buf.as_ptr() as *const _, buf.len()) } {
				-1 => Err(Error::last_os_error()),
				n => Ok(n as usize),
			}
		}
		#[cfg(windows)]
		{
			use windows_sys::Win32::Storage::FileSystem::WriteFile;
			use yazi_shim::bool_ok;

			let mut len = 0;
			bool_ok(unsafe {
				WriteFile(self.inner, buf.as_ptr(), buf.len() as u32, &mut len, ptr::null_mut())
			})?;
			Ok(len as usize)
		}
	}

	fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

#[cfg(unix)]
impl std::os::fd::AsFd for Handle {
	fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
		unsafe { std::os::fd::BorrowedFd::borrow_raw(self.inner) }
	}
}

#[cfg(unix)]
impl std::os::fd::AsRawFd for Handle {
	fn as_raw_fd(&self) -> std::os::fd::RawFd { self.inner }
}

#[cfg(unix)]
impl std::os::fd::IntoRawFd for Handle {
	fn into_raw_fd(mut self) -> std::os::fd::RawFd {
		self.close = false;
		self.inner
	}
}

#[cfg(windows)]
impl std::os::windows::io::AsRawHandle for Handle {
	fn as_raw_handle(&self) -> std::os::windows::io::RawHandle { self.inner }
}

#[cfg(windows)]
impl std::os::windows::io::IntoRawHandle for Handle {
	fn into_raw_handle(mut self) -> std::os::windows::io::RawHandle {
		self.close = false;
		self.inner
	}
}

#[cfg(unix)]
impl Handle {
	pub fn new(out: bool) -> Self {
		use std::{fs::OpenOptions, os::fd::IntoRawFd};

		use libc::{STDIN_FILENO, STDOUT_FILENO};

		let resort = Self { inner: if out { STDOUT_FILENO } else { STDIN_FILENO }, close: false };
		if unsafe { libc::isatty(resort.inner) } == 1 {
			return resort;
		}

		match OpenOptions::new().read(!out).write(out).open("/dev/tty") {
			Ok(f) => Self { inner: f.into_raw_fd(), close: true },
			Err(err) => {
				error!("Failed to open /dev/tty, falling back to stdin/stdout: {err}");
				resort
			}
		}
	}

	pub(super) fn poll(&mut self, timeout: Duration) -> std::io::Result<bool> {
		let mut tv = libc::timeval {
			tv_sec:  timeout.as_secs() as libc::time_t,
			tv_usec: timeout.subsec_micros() as libc::suseconds_t,
		};

		let result = unsafe {
			let mut set: libc::fd_set = std::mem::zeroed();
			libc::FD_ZERO(&mut set);
			libc::FD_SET(self.inner, &mut set);
			libc::select(self.inner + 1, &mut set, ptr::null_mut(), ptr::null_mut(), &mut tv)
		};

		match result {
			-1 => Err(Error::last_os_error()),
			0 => Ok(false),
			_ => Ok(true),
		}
	}

	pub fn try_clone(&self) -> std::io::Result<Self> {
		match unsafe { libc::dup(self.inner) } {
			-1 => Err(Error::last_os_error()),
			fd => Ok(Self { inner: fd, close: true }),
		}
	}
}

#[cfg(windows)]
impl Handle {
	pub fn new(out: bool) -> Self {
		use std::{io::{Error, stdin, stdout}, os::windows::io::AsRawHandle};

		use windows_sys::Win32::{Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE}, Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING}};

		let name: Vec<u16> = if out { "CONOUT$\0" } else { "CONIN$\0" }.encode_utf16().collect();
		let result = unsafe {
			CreateFileW(
				name.as_ptr(),
				GENERIC_READ | GENERIC_WRITE,
				FILE_SHARE_READ | FILE_SHARE_WRITE,
				ptr::null_mut(),
				OPEN_EXISTING,
				0,
				ptr::null_mut(),
			)
		};

		if result != INVALID_HANDLE_VALUE {
			return Self { inner: result, close: true };
		}

		error!(
			"Failed to open {}, falling back to stdin/stdout: {}",
			if out { "CONOUT$" } else { "CONIN$" },
			Error::last_os_error()
		);
		Self {
			inner: if out { stdout().as_raw_handle() } else { stdin().as_raw_handle() },
			close: false,
		}
	}

	pub(super) fn poll(&mut self, timeout: Duration) -> std::io::Result<bool> {
		use windows_sys::Win32::{Foundation::{WAIT_FAILED, WAIT_OBJECT_0}, System::Threading::WaitForSingleObject};

		let millis = timeout.as_millis();
		match unsafe { WaitForSingleObject(self.inner, millis as u32) } {
			WAIT_FAILED => Err(Error::last_os_error()),
			WAIT_OBJECT_0 => Ok(true),
			_ => Ok(false),
		}
	}

	pub fn try_clone(&self) -> std::io::Result<Self> {
		use windows_sys::Win32::{Foundation::{DUPLICATE_SAME_ACCESS, DuplicateHandle, HANDLE}, System::Threading::GetCurrentProcess};

		let proc = unsafe { GetCurrentProcess() };
		let mut handle = ptr::null_mut();
		let status = unsafe {
			DuplicateHandle(
				proc,
				self.inner,
				proc,
				&mut handle as *mut HANDLE,
				0,
				0,
				DUPLICATE_SAME_ACCESS,
			)
		};

		if status == 0 { Err(Error::last_os_error()) } else { Ok(Self { inner: handle, close: true }) }
	}
}
