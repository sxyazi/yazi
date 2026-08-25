use std::io::BufWriter;

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
}
