use std::io::{BufWriter, Write};

use anyhow::Result;
use arc_swap::ArcSwap;
use yazi_macro::writef;
use yazi_shim::cell::RoCell;
use yazi_term::TERM;
use yazi_tty::{Handle, TTY, sequence::{HideCursor, If, KittyGraphicsQuery, MoveTo, QueryOSC5522, RequestBgColor, RequestCellPixelSize, RequestDA1, RequestXtVersion, RestoreCursorPos, SaveCursorPos, SetFg, SetSgr, ShowCursor}};

use crate::{Brand, Mux};

pub static EMULATOR: RoCell<ArcSwap<Emulator>> = RoCell::new();

#[derive(Clone, Debug, Default)]
pub struct Emulator {
	pub kind:      Either<Brand, Unknown>,
	pub version:   String,
	pub light:     bool,
	pub csi_16t:   (u16, u16),
	pub force_16t: bool,
	pub osc_5522:  bool,
}

impl Default for Emulator {
	fn default() -> Self {
		Self {
			kind:      Either::Right(Unknown::default()),
			version:   String::new(),
			light:     false,
			csi_16t:   (0, 0),
			force_16t: false,
			osc_5522:  false,
		}
	}
}

impl Emulator {
	pub(super) fn from_env() -> Self {
		Self { brand: Brand::from_env().unwrap_or(Brand::Unknown), ..Default::default() }
	}

		let resort = Brand::from_env();
		writef!(
			TTY.writer(),
			"{SaveCursorPos}{}{}{}{}{}{}{RestoreCursorPos}",
			If(resort.is_none(), Mux::wrap(KittyGraphicsQuery)),
			Mux::wrap(RequestXtVersion),
			RequestCellPixelSize,
			RequestBgColor,
			QueryOSC5522,
			Mux::wrap(RequestDA1),
		)?;

	pub const fn light(&self) -> Option<bool> {
		if let Some(light) = self.color_scheme {
			Some(light)
		} else if let Some([r, g, b]) = self.background {
			let luma =
				r as f32 * 0.2627 / 65535.0 + g as f32 * 0.6780 / 65535.0 + b as f32 * 0.0593 / 65535.0;
			Some(luma > 0.6)
		} else {
			Either::Right(Unknown {
				kgp:   resp.contains("\x1b_Gi=31;OK"),
				sixel: ["?4;", "?4c", ";4;", ";4c"].iter().any(|s| resp.contains(s)),
			})
		};

		let csi_16t = Self::csi_16t(&resp).unwrap_or_default();

		let osc_5522 =
			["\x1b[?5522;1$y", "\x1b[?5522;2$y", "\x1b[?5522;3$y"].iter().any(|s| resp.contains(s));

		Ok(Self {
			kind,
			version: Self::csi_gt_q(&resp).unwrap_or_default(),
			light: Self::light_bg(&resp).unwrap_or_default(),
			csi_16t,
			force_16t: Self::force_16t(csi_16t),
			osc_5522,
		})
	}

	pub fn move_lock<F, T>((x, y): (u16, u16), cb: F) -> Result<T>
	where
		F: FnOnce(&mut BufWriter<Handle>) -> Result<T>,
	{
		use std::{thread, time::Duration};

		let mut w = TTY.lockout();
		let tmux = EMULATOR.load().mux.is_some();

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
