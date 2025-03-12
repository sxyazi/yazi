use std::{io::{Error, ErrorKind, Read, Write}, time::Duration};

use tracing::error;

pub struct Handle {
	#[cfg(unix)]
	inner:           std::os::fd::RawFd,
	#[cfg(windows)]
	inner:           std::os::windows::io::RawHandle,
	close:           bool,
	#[cfg(windows)]
	out_utf8:        bool,
	#[cfg(windows)]
	incomplete_utf8: super::IncompleteUtf8,
}

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

impl Read for Handle {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		#[cfg(unix)]
		{
			use std::os::{fd::IntoRawFd, unix::io::FromRawFd};
			let mut f = unsafe { std::fs::File::from_raw_fd(self.inner) };
			let result = f.read(buf);
			_ = f.into_raw_fd();
			result
		}
		#[cfg(windows)]
		{
			use std::os::windows::io::{FromRawHandle, IntoRawHandle};
			let mut f = unsafe { std::fs::File::from_raw_handle(self.inner) };
			let result = f.read(buf);
			_ = f.into_raw_handle();
			result
		}
	}
}

impl Write for Handle {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		#[cfg(unix)]
		{
			use std::os::{fd::IntoRawFd, unix::io::FromRawFd};
			let mut f = unsafe { std::fs::File::from_raw_fd(self.inner) };
			let result = f.write(buf);
			_ = f.into_raw_fd();
			result
		}
		#[cfg(windows)]
		{
			use std::os::windows::io::{FromRawHandle, IntoRawHandle};
			if self.out_utf8 {
				let mut f = unsafe { std::fs::File::from_raw_handle(self.inner) };
				let result = f.write(buf);
				_ = f.into_raw_handle();
				result
			} else {
				super::write_console_utf16(buf, &mut self.incomplete_utf8, self.inner)
			}
		}
	}

	fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

#[cfg(unix)]
impl Handle {
	pub(super) fn new(out: bool) -> Self {
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
			libc::select(self.inner + 1, &mut set, std::ptr::null_mut(), std::ptr::null_mut(), &mut tv)
		};

		match result {
			-1 => Err(Error::last_os_error()),
			0 => Ok(false),
			_ => Ok(true),
		}
	}

	pub(super) fn read_u8(&mut self) -> std::io::Result<u8> {
		let mut b = 0;
		match unsafe { libc::read(self.inner, &mut b as *mut _ as *mut _, 1) } {
			-1 => Err(Error::last_os_error()),
			0 => Err(Error::new(ErrorKind::UnexpectedEof, "unexpected EOF")),
			_ => Ok(b),
		}
	}
}

#[cfg(windows)]
impl Handle {
	pub(super) fn new(out: bool) -> Self {
		use std::{io::{Error, stdin, stdout}, os::windows::io::AsRawHandle};

		use windows_sys::Win32::{Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE}, Globalization::CP_UTF8, Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING}, System::Console::GetConsoleOutputCP};

		let name: Vec<u16> = if out { "CONOUT$\0" } else { "CONIN$\0" }.encode_utf16().collect();
		let result = unsafe {
			CreateFileW(
				name.as_ptr(),
				GENERIC_READ | GENERIC_WRITE,
				FILE_SHARE_READ | FILE_SHARE_WRITE,
				std::ptr::null_mut(),
				OPEN_EXISTING,
				0,
				std::ptr::null_mut(),
			)
		};

		if result != INVALID_HANDLE_VALUE {
			return Self {
				inner:           result,
				close:           true,
				out_utf8:        unsafe { GetConsoleOutputCP() } == CP_UTF8,
				incomplete_utf8: Default::default(),
			};
		}

		error!(
			"Failed to open {}, falling back to stdin/stdout: {}",
			if out { "CONOUT$" } else { "CONIN$" },
			Error::last_os_error()
		);
		Self {
			inner:           if out { stdout().as_raw_handle() } else { stdin().as_raw_handle() },
			close:           false,
			out_utf8:        unsafe { GetConsoleOutputCP() } == CP_UTF8,
			incomplete_utf8: Default::default(),
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

	pub(super) fn read_u8(&mut self) -> std::io::Result<u8> {
		use windows_sys::Win32::Storage::FileSystem::ReadFile;

		let mut buf = 0;
		let mut bytes = 0;
		let success = unsafe { ReadFile(self.inner, &mut buf, 1, &mut bytes, std::ptr::null_mut()) };

		if success == 0 {
			return Err(Error::last_os_error());
		} else if bytes == 0 {
			return Err(Error::new(ErrorKind::UnexpectedEof, "unexpected EOF"));
		}
		Ok(buf)
	}
}
