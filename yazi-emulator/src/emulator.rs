use std::io::{BufWriter, Write};

use anyhow::Result;
use arc_swap::ArcSwap;
use tracing::debug;
use yazi_macro::writef;
use yazi_shim::cell::RoCell;
use yazi_term::{TERM, event::Report};
use yazi_tty::{Handle, TTY, sequence::{HideCursor, MoveTo, RestoreCursorPos, SaveCursorPos, ShowCursor}};

use crate::Brand;

pub static EMULATOR: RoCell<ArcSwap<Emulator>> = RoCell::new();

#[derive(Clone, Debug, Default)]
pub struct Emulator {
	pub brand:        Brand,
	pub version:      String,
	pub kgp:          bool,
	pub sixel:        bool,
	pub light:        bool,
	pub csi_16t:      (u16, u16),
	pub force_16t:    bool,
	pub cursor_blink: bool,
	pub cursor_shape: Option<u8>,
	pub tmux:         bool,
}

impl Emulator {
	pub(super) fn from_env() -> Self {
		Self { brand: Brand::from_env().unwrap_or(Brand::Unknown), ..Default::default() }
	}

	pub fn apply(&mut self, report: &Report) {
		match report {
			Report::CursorBlink(blink) => self.cursor_blink = *blink,
			Report::CursorShape(shape) => self.cursor_shape = Some(*shape),
			Report::Da1(attrs) => self.sixel = attrs.contains(&4),
			Report::XtVersion(version) => {
				self.version = version.to_string();
				if let Some(brand) = Brand::from_csi(version) {
					self.brand = brand;
				}
			}
			Report::CellPixelSize { width, height } => {
				self.csi_16t = (*width, *height);
				self.force_16t = Self::force_16t(self.csi_16t);
			}
			Report::BackgroundColor([r, g, b]) => {
				let luma = *r as f32 * 0.2627 / 65535.0
					+ *g as f32 * 0.6780 / 65535.0
					+ *b as f32 * 0.0593 / 65535.0;
				debug!("Detected background color: {r:04x}/{g:04x}/{b:04x} (luma = {luma:.2})");
				self.light = luma > 0.6;
			}
			Report::KittyGraphics { id: 31, ok } => self.kgp = *ok,
			_ => {}
		}
	}

	pub fn move_lock<F, T>((x, y): (u16, u16), cb: F) -> Result<T>
	where
		F: FnOnce(&mut BufWriter<Handle>) -> Result<T>,
	{
		use std::{thread, time::Duration};

		let mut w = TTY.lockout();
		let tmux = EMULATOR.load().tmux;

		// I really don't want to add this,
		// But tmux and ConPTY sometimes cause the cursor position to get out of sync.
		if tmux || cfg!(windows) {
			writef!(w, "{SaveCursorPos}{}{ShowCursor}", MoveTo(x, y))?;
			writef!(w, "{}{ShowCursor}", MoveTo(x, y))?;
			writef!(w, "{}{ShowCursor}", MoveTo(x, y))?;
			thread::sleep(Duration::from_millis(1));
		} else {
			write!(w, "{SaveCursorPos}{}", MoveTo(x, y))?;
		}

		let result = cb(&mut w);
		if tmux || cfg!(windows) {
			write!(w, "{HideCursor}{RestoreCursorPos}")?;
		} else {
			write!(w, "{RestoreCursorPos}")?;
		}

		w.flush()?;
		result
	}

	fn force_16t((w, h): (u16, u16)) -> bool {
		if w == 0 || h == 0 {
			return false;
		}

		TERM.dimension().ratio().is_none_or(|(rw, rh)| rw.floor() as u16 != w || rh.floor() as u16 != h)
	}
}
