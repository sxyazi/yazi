use std::mem;

use crossterm::terminal::WindowSize;

use crate::EMULATOR;

#[derive(Clone, Copy, Debug)]
pub struct Dimension {
	pub rows:    u16,
	pub columns: u16,
	pub width:   u16,
	pub height:  u16,
}

impl From<WindowSize> for Dimension {
	fn from(s: WindowSize) -> Self {
		Self { rows: s.rows, columns: s.columns, width: s.width, height: s.height }
	}
}

impl From<Dimension> for WindowSize {
	fn from(d: Dimension) -> Self {
		Self { rows: d.rows, columns: d.columns, width: d.width, height: d.height }
	}
}

impl Dimension {
	pub fn available() -> Self {
		let mut size = WindowSize { rows: 0, columns: 0, width: 0, height: 0 };
		if let Ok(s) = crossterm::terminal::window_size() {
			_ = mem::replace(&mut size, s);
		}

		if size.columns == 0 || size.rows == 0 {
			if let Ok((cols, rows)) = crossterm::terminal::size() {
				size.columns = cols;
				size.rows = rows;
			}
		}

		size.into()
	}

	pub fn ratio(self) -> Option<(f64, f64)> {
		if self.rows == 0 || self.columns == 0 || self.width == 0 || self.height == 0 {
			None
		} else {
			Some((self.width as f64 / self.columns as f64, self.height as f64 / self.rows as f64))
		}
	}

	pub fn cell_size() -> Option<(f64, f64)> {
		let emu = EMULATOR.get();
		Some(if emu.force_16t {
			(emu.csi_16t.0 as f64, emu.csi_16t.1 as f64)
		} else if let Some(r) = Self::available().ratio() {
			r
		} else if emu.csi_16t.0 != 0 && emu.csi_16t.1 != 0 {
			(emu.csi_16t.0 as f64, emu.csi_16t.1 as f64)
		} else {
			None?
		})
	}
}
