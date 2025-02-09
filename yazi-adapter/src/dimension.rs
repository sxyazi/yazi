use std::mem;

use crossterm::terminal::WindowSize;

use crate::EMULATOR;

pub struct Dimension;

impl Dimension {
	pub fn available() -> WindowSize {
		let mut size = WindowSize { rows: 0, columns: 0, width: 0, height: 0 };
		if let Ok(s) = crossterm::terminal::window_size() {
			_ = mem::replace(&mut size, s);
		}

		if size.rows == 0 || size.columns == 0 {
			if let Ok((cols, rows)) = crossterm::terminal::size() {
				size.columns = cols;
				size.rows = rows;
			}
		}

		// TODO: Use `CSI 14 t` to get the actual size of the terminal
		// if size.width == 0 || size.height == 0 {}

		size
	}

	#[inline]
	pub fn ratio() -> Option<(f64, f64)> {
		let s = Self::available();
		Some(if s.width == 0 || s.height == 0 {
			let s = EMULATOR.get().cell_size?;
			(s.0 as f64, s.1 as f64)
		} else {
			(f64::from(s.width) / f64::from(s.columns), f64::from(s.height) / f64::from(s.rows))
		})
	}
}
