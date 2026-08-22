use std::{fmt::Display, io::{BufWriter, Error, ErrorKind, Read}, sync::atomic::{AtomicBool, Ordering}, time::{Duration, Instant}};

use parking_lot::{Mutex, MutexGuard};

use super::Handle;
use crate::{TtyReader, TtyWriter, sequence::TmuxPassthrough};

pub struct Tty {
	stdin:  Mutex<Handle>,
	stdout: Mutex<BufWriter<Handle>>,
	tmux:   AtomicBool,
}

impl Default for Tty {
	fn default() -> Self {
		Self {
			stdin:  Mutex::new(Handle::new(false)),
			stdout: Mutex::new(BufWriter::new(Handle::new(true))),
			tmux:   AtomicBool::new(false),
		}
	}
}

impl Tty {
	pub const fn reader(&self) -> TtyReader<'_> { TtyReader(&self.stdin) }

	pub const fn writer(&self) -> TtyWriter<'_> { TtyWriter(&self.stdout) }

	pub fn lockin(&self) -> MutexGuard<'_, Handle> { self.stdin.lock() }

	pub fn lockout(&self) -> MutexGuard<'_, BufWriter<Handle>> { self.stdout.lock() }

	pub fn enable_tmux_passthrough(&self) { self.tmux.store(true, Ordering::Relaxed); }

	pub fn tmux_passthrough<T: Display>(&self, sequence: T) -> TmuxPassthrough<T> {
		TmuxPassthrough(sequence, self.tmux.load(Ordering::Relaxed))
	}

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
}
