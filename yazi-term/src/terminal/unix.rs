use std::{env, io, sync::Arc};

use rustix::termios;
use yazi_tty::Tty;

use crate::{Dimension, restorer::Restorer, source::EventSource};

pub struct Terminal<'a> {
	tty:          &'a Tty,
	pub source:   Arc<EventSource<'a>>,
	pub restorer: Restorer,
}

impl Drop for Terminal<'_> {
	fn drop(&mut self) { self.restorer.restore(self.tty); }
}

impl<'a> Terminal<'a> {
	pub fn new(tty: &'a Tty) -> io::Result<Self> {
		let source = Arc::new(EventSource::new(tty.reader(), tty.writer())?);
		let restorer = Restorer::new(tty)?;

		let term = Self { tty, source, restorer };
		term.setup()?;

		Ok(term)
	}

	pub fn setup(&self) -> io::Result<()> { Ok(()) }

	pub fn dimension(&self) -> Dimension {
		let mut dim = Dimension::default();
		if let Ok(size) = termios::tcgetwinsize(self.tty.writer()) {
			dim = size.into();
		}

		if dim.cols == 0 || dim.rows == 0 {
			if let Ok(Ok(n)) = env::var("LINES").map(|l| l.parse()) {
				dim.rows = n;
			}
			if let Ok(Ok(n)) = env::var("COLUMNS").map(|c| c.parse()) {
				dim.cols = n;
			}
		}

		dim
	}

	pub fn enter_raw_mode(&self) -> io::Result<()> {
		let mut termios = self.restorer.termios.clone();
		termios.make_raw();

		termios::tcsetattr(self.tty.writer(), termios::OptionalActions::Flush, &termios)?;
		Ok(())
	}

	pub fn enter_cooked_mode(&self) -> io::Result<()> {
		termios::tcsetattr(self.tty.writer(), termios::OptionalActions::Now, &self.restorer.termios)?;
		Ok(())
	}
}
