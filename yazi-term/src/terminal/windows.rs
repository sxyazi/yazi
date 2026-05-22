use std::{io, mem, os::windows::prelude::*, sync::Arc};

use windows_sys::Win32::{Globalization::CP_UTF8, System::Console::{CONSOLE_MODE, CONSOLE_SCREEN_BUFFER_INFO, DISABLE_NEWLINE_AUTO_RETURN, ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_MOUSE_INPUT, ENABLE_PROCESSED_INPUT, ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING, ENABLE_WINDOW_INPUT, GetConsoleScreenBufferInfo, SetConsoleCP, SetConsoleMode, SetConsoleOutputCP}};
use yazi_shim::bool_ok;
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
		let source = Arc::new(EventSource::new(tty.reader())?);
		let restorer = Restorer::new(tty)?;

		let term = Self { tty, source, restorer };
		term.setup()?;

		Ok(term)
	}

	pub fn setup(&self) -> io::Result<()> {
		self.set_input_cp(CP_UTF8)?;
		self.set_output_cp(CP_UTF8)?;
		Ok(())
	}

	pub fn dimension(&self) -> Dimension {
		let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { mem::zeroed() };
		if unsafe { GetConsoleScreenBufferInfo(self.tty.writer().as_raw_handle(), &mut info) } == 0 {
			return Dimension::default();
		}

		Dimension {
			rows:   (info.srWindow.Bottom - info.srWindow.Top) as u16 + 1,
			cols:   (info.srWindow.Right - info.srWindow.Left) as u16 + 1,
			width:  0,
			height: 0,
		}
	}

	pub fn enter_raw_mode(&self) -> io::Result<()> {
		self.set_input_mode(
			(self.restorer.input_mode
				& !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT))
				| ENABLE_VIRTUAL_TERMINAL_INPUT
				| ENABLE_MOUSE_INPUT
				| ENABLE_WINDOW_INPUT,
		)?;

		self.set_output_mode(
			self.restorer.output_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING | DISABLE_NEWLINE_AUTO_RETURN,
		)
	}

	pub fn enter_cooked_mode(&self) -> io::Result<()> {
		self.set_output_mode(
			self.restorer.output_mode
				& !(ENABLE_VIRTUAL_TERMINAL_PROCESSING | DISABLE_NEWLINE_AUTO_RETURN),
		)?;

		self.set_input_mode(
			(self.restorer.input_mode
				& !(ENABLE_VIRTUAL_TERMINAL_INPUT | ENABLE_MOUSE_INPUT | ENABLE_WINDOW_INPUT))
				| ENABLE_ECHO_INPUT
				| ENABLE_LINE_INPUT
				| ENABLE_PROCESSED_INPUT,
		)
	}

	fn set_input_cp(&self, cp: u32) -> io::Result<()> { bool_ok(unsafe { SetConsoleCP(cp) }) }

	fn set_output_cp(&self, cp: u32) -> io::Result<()> { bool_ok(unsafe { SetConsoleOutputCP(cp) }) }

	fn set_input_mode(&self, mode: CONSOLE_MODE) -> io::Result<()> {
		bool_ok(unsafe { SetConsoleMode(self.tty.reader().as_raw_handle(), mode) })
	}

	fn set_output_mode(&self, mode: CONSOLE_MODE) -> io::Result<()> {
		bool_ok(unsafe { SetConsoleMode(self.tty.writer().as_raw_handle(), mode) })
	}
}
