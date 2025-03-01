use std::{io::{Error, ErrorKind}, ops::Deref, time::{Duration, Instant}};

pub struct AsyncStdin {
	fd:  Fd,
	#[cfg(unix)]
	fds: libc::fd_set,
}

impl Default for AsyncStdin {
	fn default() -> Self {
		let fd = Fd::new().expect("failed to open stdin");
		#[cfg(unix)]
		{
			let mut me = Self { fd, fds: unsafe { std::mem::MaybeUninit::zeroed().assume_init() } };
			me.reset();
			me
		}
		#[cfg(windows)]
		{
			Self { fd }
		}
	}
}

impl AsyncStdin {
	pub fn read_until<P>(&mut self, timeout: Duration, predicate: P) -> (Vec<u8>, std::io::Result<()>)
	where
		P: Fn(u8, &[u8]) -> bool,
	{
		let mut buf: Vec<u8> = Vec::with_capacity(200);
		let now = Instant::now();

		let mut read = || {
			loop {
				if now.elapsed() > timeout {
					return Err(Error::new(ErrorKind::TimedOut, "timed out"));
				} else if !self.poll(Duration::from_millis(30))? {
					continue;
				}

				let b = self.read_u8()?;
				buf.push(b);

				if predicate(b, &buf) {
					break;
				}
			}
			Ok(())
		};

		let result = read();
		(buf, result)
	}
}

// --- Unix
#[cfg(unix)]
impl AsyncStdin {
	pub fn poll(&mut self, timeout: Duration) -> std::io::Result<bool> {
		let mut tv = libc::timeval {
			tv_sec:  timeout.as_secs() as libc::time_t,
			tv_usec: timeout.subsec_micros() as libc::suseconds_t,
		};

		let result = unsafe {
			libc::select(*self.fd + 1, &mut self.fds, std::ptr::null_mut(), std::ptr::null_mut(), &mut tv)
		};

		match result {
			-1 => Err(Error::last_os_error()),
			0 => Ok(false),
			_ => {
				self.reset();
				Ok(true)
			}
		}
	}

	pub fn read_u8(&mut self) -> std::io::Result<u8> {
		let mut b = 0;
		match unsafe { libc::read(*self.fd, &mut b as *mut _ as *mut _, 1) } {
			-1 => Err(Error::last_os_error()),
			0 => Err(Error::new(ErrorKind::UnexpectedEof, "unexpected EOF")),
			_ => Ok(b),
		}
	}

	fn reset(&mut self) {
		unsafe {
			libc::FD_ZERO(&mut self.fds);
			libc::FD_SET(*self.fd, &mut self.fds);
		}
	}
}

// --- Windows
#[cfg(windows)]
impl AsyncStdin {
	pub fn poll(&mut self, timeout: Duration) -> std::io::Result<bool> {
		use windows_sys::Win32::{Foundation::{WAIT_FAILED, WAIT_OBJECT_0}, System::Threading::WaitForSingleObject};

		let millis = timeout.as_millis();
		match unsafe { WaitForSingleObject(*self.fd, millis as u32) } {
			WAIT_FAILED => Err(Error::last_os_error()),
			WAIT_OBJECT_0 => Ok(true),
			_ => Ok(false),
		}
	}

	pub fn read_u8(&mut self) -> std::io::Result<u8> {
		use windows_sys::Win32::Storage::FileSystem::ReadFile;

		let mut buf = 0;
		let mut bytes = 0;
		let success = unsafe { ReadFile(*self.fd, &mut buf, 1, &mut bytes, std::ptr::null_mut()) };

		if success == 0 {
			return Err(Error::last_os_error());
		} else if bytes == 0 {
			return Err(Error::new(ErrorKind::UnexpectedEof, "unexpected EOF"));
		}
		Ok(buf)
	}
}

// --- Fd
struct Fd {
	#[cfg(unix)]
	inner: std::os::fd::RawFd,
	#[cfg(windows)]
	inner: std::os::windows::io::RawHandle,
	close: bool,
}

impl Deref for Fd {
	#[cfg(unix)]
	type Target = std::os::fd::RawFd;
	#[cfg(windows)]
	type Target = std::os::windows::io::RawHandle;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Drop for Fd {
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

impl Fd {
	#[cfg(unix)]
	fn new() -> std::io::Result<Self> {
		use std::{fs::OpenOptions, os::fd::IntoRawFd};

		Ok(if unsafe { libc::isatty(libc::STDIN_FILENO) } == 1 {
			Self { inner: libc::STDIN_FILENO, close: false }
		} else {
			Self {
				inner: OpenOptions::new().read(true).write(true).open("/dev/tty")?.into_raw_fd(),
				close: true,
			}
		})
	}

	#[cfg(windows)]
	fn new() -> std::io::Result<Self> {
		use windows_sys::Win32::{Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE}, Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING}};

		let name: Vec<u16> = "CONIN$\0".encode_utf16().collect();
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

		if result == INVALID_HANDLE_VALUE {
			Err(std::io::Error::last_os_error())
		} else {
			Ok(Self { inner: result, close: true })
		}
	}
}
