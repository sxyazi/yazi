use std::{io::{BufWriter, Error, ErrorKind, Read, Write}, time::{Duration, Instant}};

use parking_lot::{Mutex, MutexGuard};

use super::Handle;

pub struct Tty {
	stdin:  Mutex<Handle>,
	stdout: Mutex<BufWriter<Handle>>,
}

impl Default for Tty {
	fn default() -> Self {
		#[cfg(windows)]
		Self::set_code_page().expect("failed to set terminal code page");

		let stdin = Handle::new(false).expect("failed to open stdin");
		let stdout = Handle::new(true).expect("failed to open stdout");
		Self { stdin: Mutex::new(stdin), stdout: Mutex::new(BufWriter::new(stdout)) }
	}
}

impl Tty {
	pub const fn reader(&self) -> TtyReader { TtyReader(&self.stdin) }

	pub const fn writer(&self) -> TtyWriter { TtyWriter(&self.stdout) }

	pub fn lockin(&self) -> MutexGuard<Handle> { self.stdin.lock() }

	pub fn lockout(&self) -> MutexGuard<BufWriter<Handle>> { self.stdout.lock() }

	pub fn read_until<P>(&self, timeout: Duration, predicate: P) -> (Vec<u8>, std::io::Result<()>)
	where
		P: Fn(u8, &[u8]) -> bool,
	{
		let mut buf: Vec<u8> = Vec::with_capacity(200);
		let now = Instant::now();

		let mut read = || {
			let mut stdin = self.stdin.lock();
			loop {
				if now.elapsed() > timeout {
					return Err(Error::new(ErrorKind::TimedOut, "timed out"));
				} else if !stdin.poll(Duration::from_millis(30))? {
					continue;
				}

				let b = stdin.read_u8()?;
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

	#[cfg(windows)]
	fn set_code_page() -> std::io::Result<()> {
		use windows_sys::Win32::{Globalization::CP_UTF8, System::Console::SetConsoleOutputCP};

		if unsafe { SetConsoleOutputCP(CP_UTF8) } == 0 { Err(Error::last_os_error()) } else { Ok(()) }
	}
}

// --- Reader
pub struct TtyReader<'a>(&'a Mutex<Handle>);

impl Read for TtyReader<'_> {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> { self.0.lock().read(buf) }
}

// --- Writer
pub struct TtyWriter<'a>(&'a Mutex<BufWriter<Handle>>);

impl Write for TtyWriter<'_> {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { self.0.lock().write(buf) }

	fn flush(&mut self) -> std::io::Result<()> { self.0.lock().flush() }
}
