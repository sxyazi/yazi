use std::io::{self, Write};

use rustix::termios::{self, OptionalActions, Termios};
use yazi_tty::Tty;

#[derive(Clone)]
pub struct Restorer {
	pub(crate) termios: Termios,
}

impl Restorer {
	pub(crate) fn new(tty: &Tty) -> io::Result<Self> {
		Ok(Self { termios: rustix::termios::tcgetattr(tty.writer())? })
	}

	pub fn restore(&self, tty: &Tty) {
		tty.writer().flush().ok();
		termios::tcsetattr(tty.writer(), OptionalActions::Now, &self.termios).ok();
	}
}
