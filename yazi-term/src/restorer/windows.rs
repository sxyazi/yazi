use std::{io::{self, Write}, os::windows::io::AsRawHandle};

use windows_sys::Win32::System::Console::{CONSOLE_MODE, GetConsoleCP, GetConsoleMode, GetConsoleOutputCP, SetConsoleCP, SetConsoleMode, SetConsoleOutputCP};
use yazi_shim::{bool_ok, nz_ok};
use yazi_tty::Tty;

#[derive(Clone)]
pub struct Restorer {
	pub(crate) input_mode:  CONSOLE_MODE,
	pub(crate) output_mode: CONSOLE_MODE,
	pub(crate) input_cp:    u32,
	pub(crate) output_cp:   u32,
}

impl Restorer {
	pub(crate) fn new(tty: &Tty) -> io::Result<Self> {
		let mut input_mode = 0;
		bool_ok(unsafe { GetConsoleMode(tty.reader().as_raw_handle(), &mut input_mode) })?;

		let mut output_mode = 0;
		bool_ok(unsafe { GetConsoleMode(tty.writer().as_raw_handle(), &mut output_mode) })?;

		let input_cp = nz_ok(unsafe { GetConsoleCP() })?;
		let output_cp = nz_ok(unsafe { GetConsoleOutputCP() })?;

		Ok(Self { input_mode, output_mode, input_cp, output_cp })
	}

	pub fn restore(&self, tty: &Tty) {
		tty.writer().flush().ok();
		unsafe {
			SetConsoleCP(self.input_cp);
			SetConsoleMode(tty.reader().as_raw_handle(), self.input_mode);
			SetConsoleOutputCP(self.output_cp);
			SetConsoleMode(tty.writer().as_raw_handle(), self.output_mode);
		}
	}
}
