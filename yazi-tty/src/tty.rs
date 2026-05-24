use std::{io::{BufWriter, Error, ErrorKind, Read}, time::{Duration, Instant}};

use parking_lot::{Mutex, MutexGuard};

use super::Handle;
use crate::{TtyReader, TtyWriter};

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
					return Err(Error::from(ErrorKind::TimedOut));
				} else if !stdin.poll(Duration::from_millis(30))? {
					continue;
				}

				let mut b = [0u8];
				stdin.read_exact(&mut b)?;
				buf.push(b[0]);

				if predicate(b[0], &buf) {
					break;
				}
			}
			Ok(())
		};

		let result = read();
		(buf, result)
	}

	pub fn drain_until_quiet(&self, timeout: Duration, quiet: Duration) -> std::io::Result<usize> {
		let mut drained = 0;
		let until = Instant::now() + timeout;
		let mut stdin = self.stdin.lock();

		while Instant::now() < until {
			match stdin.poll(quiet) {
				Ok(true) => {}
				Ok(false) => break,
				Err(e) if e.kind() == ErrorKind::Interrupted => continue,
				Err(e) => return Err(e),
			}

			let mut b = [0u8];
			loop {
				match stdin.read_exact(&mut b) {
					Ok(()) => {
						drained += 1;
						break;
					}
					Err(e) if e.kind() == ErrorKind::Interrupted => continue,
					Err(e) => return Err(e),
				}
			}
		}
		Ok(drained)
	}
}
