use std::{io::{BufWriter, Error, ErrorKind, Read, Write}, time::{Duration, Instant}};

use parking_lot::{Mutex, MutexGuard};

use super::Handle;

pub struct Tty {
	stdin:  Mutex<Handle>,
	stdout: Mutex<BufWriter<Handle>>,
}

impl Default for Tty {
	fn default() -> Self {
		Self {
			stdin:  Mutex::new(Handle::new(false)),
			stdout: Mutex::new(BufWriter::new(Handle::new(true))),
		}
	}
}

impl Tty {
	pub const fn reader(&self) -> TtyReader<'_> { TtyReader(&self.stdin) }

	pub const fn writer(&self) -> TtyWriter<'_> { TtyWriter(&self.stdout) }

	pub fn lockin(&self) -> MutexGuard<'_, Handle> { self.stdin.lock() }

	pub fn lockout(&self) -> MutexGuard<'_, BufWriter<Handle>> { self.stdout.lock() }

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
}

// --- Reader
pub struct TtyReader<'a>(&'a Mutex<Handle>);

impl Read for TtyReader<'_> {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> { self.0.lock().read(buf) }
}

// --- Writer
pub struct TtyWriter<'a>(&'a Mutex<BufWriter<Handle>>);

impl std::io::Write for TtyWriter<'_> {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { self.0.lock().write(buf) }

	fn flush(&mut self) -> std::io::Result<()> { self.0.lock().flush() }
}

impl std::fmt::Write for TtyWriter<'_> {
	fn write_str(&mut self, s: &str) -> std::fmt::Result {
		self.0.lock().write_all(s.as_bytes()).map_err(|_| std::fmt::Error)
	}
}
