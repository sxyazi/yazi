use std::io::{BufWriter, Write};

use anyhow::Result;
use arc_swap::ArcSwap;
use yazi_macro::writef;
use yazi_shim::cell::{RoCell, SyncCell};
use yazi_term::{TERM, event::Report};
use yazi_tty::{Handle, TTY, sequence::{EnterAlternateScreen, HideCursor, LeaveAlternateScreen, MoveTo, RestoreCursorPos, SaveCursorPos, ShowCursor}};

use crate::{Brand, Mux, Probe};

pub static EMULATOR: RoCell<Emulator> = RoCell::new();

#[derive(Debug, Default)]
pub struct Emulator {
	pub brand:        SyncCell<Brand>,
	pub version:      ArcSwap<String>,
	pub csi_u:        SyncCell<Option<u8>>,
	pub kgp:          SyncCell<bool>,
	pub sixel:        SyncCell<bool>,
	pub background:   SyncCell<Option<[u16; 3]>>,
	pub color_scheme: SyncCell<Option<bool>>,
	pub csi_16t:      SyncCell<(u16, u16)>,
	pub force_16t:    SyncCell<bool>,
	pub osc_5522:     SyncCell<bool>,
	pub cursor_blink: SyncCell<bool>,
	pub cursor_shape: SyncCell<Option<u8>>,
	pub mux:          SyncCell<Option<Mux>>,

	pub probe:          Probe,
	pub(super) started: SyncCell<bool>,
}

impl Emulator {
	pub(super) fn from_env() -> Self {
		Self { brand: Brand::from_env().unwrap_or(Brand::Unknown).into(), ..Default::default() }
	}

	pub fn start(&self) -> Result<()> {
		if self.started.replace(true) {
			return Ok(());
		}

		TERM.setup()?;
		TERM.enter_raw_mode()?;
		writef!(TTY.writer(), "{EnterAlternateScreen}")?;

		self.probe.reset();
		self.request()
	}

	pub fn stop(&self) {
		if !self.started.replace(false) {
			return;
		}

		writef!(TTY.writer(), "{LeaveAlternateScreen}").ok();
		TERM.source.wake().ok();
		TERM.restorer.restore(&TTY);
	}

	pub fn restart(&self) -> Result<()> {
		self.mux.set(Some(Mux { sixel: self.sixel.get() }));
		TTY.enable_tmux_passthrough();

		// Only these requests are passed through tmux after restarting.
		self.brand.set(Brand::Unknown);
		self.version.store(Default::default());
		self.kgp.set(false);
		self.sixel.set(false);

		self.probe.reset();
		self.request()
	}

	pub fn apply(&self, report: &Report) {
		match report {
			Report::CsiU(flags) => self.csi_u.set(Some(*flags)),
			Report::CursorBlink(blink) => self.cursor_blink.set(*blink),
			Report::CursorShape(shape) => self.cursor_shape.set(Some(*shape)),
			Report::Da1(attrs) => self.sixel.set(attrs.contains(&4)),
			Report::XtVersion(version) => {
				self.version.store(version.to_string().into());
				if let Some(brand) = Brand::from_csi(version) {
					self.brand.set(brand);
				}
			}
			Report::CellPixelSize { width, height } => {
				self.csi_16t.set((*width, *height));
				self.force_16t.set(Self::force_16t((*width, *height)));
			}
			Report::BackgroundColor(rgb) => self.background.set(Some(*rgb)),
			Report::ColorScheme(light) => self.color_scheme.set(Some(*light)),
			Report::KittyGraphics { id: 31, ok } => self.kgp.set(*ok),
			Report::Clipboard(supported) => self.osc_5522.set(*supported),
			_ => {}
		}
	}

	pub fn light(&self) -> Option<bool> {
		if let Some(light) = self.color_scheme.get() {
			Some(light)
		} else if let Some([r, g, b]) = self.background.get() {
			let luma =
				r as f32 * 0.2627 / 65535.0 + g as f32 * 0.6780 / 65535.0 + b as f32 * 0.0593 / 65535.0;
			Some(luma > 0.6)
		} else {
			None
		}
	}

	pub fn move_lock<F, T>((x, y): (u16, u16), cb: F) -> Result<T>
	where
		F: FnOnce(&mut BufWriter<Handle>) -> Result<T>,
	{
		use std::{thread, time::Duration};

		let mut w = TTY.lockout();
		let tmux = EMULATOR.mux.get().is_some();

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

	pub fn needs_passthrough(&self) -> bool {
		self.brand.get() == Brand::Tmux && self.mux.get().is_none()
	}

	fn force_16t((w, h): (u16, u16)) -> bool {
		if w == 0 || h == 0 {
			return false;
		}

		TERM.dimension().ratio().is_none_or(|(rw, rh)| rw.floor() as u16 != w || rh.floor() as u16 != h)
	}
}
