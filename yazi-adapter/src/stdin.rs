use std::{io::{Error, ErrorKind}, time::{Duration, Instant}};

pub struct AsyncStdin {
	#[cfg(unix)]
	fds: libc::fd_set,
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

				let b = Self::read_u8()?;
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

#[cfg(unix)]
impl Default for AsyncStdin {
	fn default() -> Self {
		let mut me = Self { fds: unsafe { std::mem::MaybeUninit::zeroed().assume_init() } };
		me.reset();
		me
	}
}

#[cfg(unix)]
impl AsyncStdin {
	pub fn poll(&mut self, timeout: Duration) -> std::io::Result<bool> {
		let mut tv = libc::timeval {
			tv_sec:  timeout.as_secs() as libc::time_t,
			tv_usec: timeout.subsec_micros() as libc::suseconds_t,
		};

		let result = unsafe {
			libc::select(
				libc::STDIN_FILENO + 1,
				&mut self.fds,
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				&mut tv,
			)
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

	pub fn read_u8() -> std::io::Result<u8> {
		let mut b = 0;
		match unsafe { libc::read(libc::STDIN_FILENO, &mut b as *mut _ as *mut _, 1) } {
			-1 => Err(Error::last_os_error()),
			0 => Err(Error::new(ErrorKind::UnexpectedEof, "unexpected EOF")),
			_ => Ok(b),
		}
	}

	fn reset(&mut self) {
		unsafe {
			libc::FD_ZERO(&mut self.fds);
			libc::FD_SET(libc::STDIN_FILENO, &mut self.fds);
		}
	}
}

#[cfg(windows)]
impl Default for AsyncStdin {
	fn default() -> Self { Self {} }
}

#[cfg(windows)]
impl AsyncStdin {
	pub fn poll(&mut self, timeout: Duration) -> std::io::Result<bool> {
		use std::os::windows::io::AsRawHandle;

		use windows_sys::Win32::{Foundation::{WAIT_FAILED, WAIT_OBJECT_0}, System::Threading::WaitForSingleObject};

		let handle = std::io::stdin().as_raw_handle();
		let millis = timeout.as_millis();
		match unsafe { WaitForSingleObject(handle, millis as u32) } {
			WAIT_FAILED => Err(Error::last_os_error()),
			WAIT_OBJECT_0 => Ok(true),
			_ => Ok(false),
		}
	}

	pub fn read_u8() -> std::io::Result<u8> {
		use std::os::windows::io::AsRawHandle;

		use windows_sys::Win32::Storage::FileSystem::ReadFile;

		let mut buf = 0;
		let mut bytes = 0;
		let success = unsafe {
			ReadFile(std::io::stdin().as_raw_handle(), &mut buf, 1, &mut bytes, std::ptr::null_mut())
		};

		if success == 0 {
			return Err(Error::last_os_error());
		} else if bytes == 0 {
			return Err(Error::new(ErrorKind::UnexpectedEof, "unexpected EOF"));
		}
		Ok(buf)
	}
}
