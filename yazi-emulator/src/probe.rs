use std::{fmt::Display, time::Duration};

use anyhow::{Result, bail};
use tokio::time;
use yazi_macro::{error, writef};
use yazi_shared::id::Ids;
use yazi_term::{TERM, event::{Event, Report}, stream::EventStream};
use yazi_tty::{TTY, sequence::{EnterAlternateScreen, LeaveAlternateScreen, ProbeClipboard, RequestBgColor, RequestCellPixelSize, RequestColorScheme, RequestCsiU, RequestCursorBlink, RequestCursorStyle, RequestDA1, RequestKittyGraphics, RequestXtVersion, RestoreCursorPos, SaveCursorPos, TmuxPassthrough}};

use crate::{Brand, Emulator, Mux};

static IDS: Ids = Ids::new();

impl Emulator {
	pub fn start(&self) -> Result<()> {
		if self.started.replace(true) {
			return Ok(());
		}

		TERM.setup()?;
		TERM.enter_raw_mode()?;
		writef!(TTY.writer(), "{EnterAlternateScreen}")?;

		self.probe_id.set(IDS.next());
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
		self.probe_id.set(IDS.next());

		// Only these requests are passed through tmux after restarting.
		self.brand.set(Brand::Unknown);
		self.version.store(Default::default());
		self.kgp.set(false);
		self.sixel.set(false);

		self.request()
	}

	pub async fn probe() -> Result<Self> {
		TERM.enter_raw_mode()?;
		let mut stream = EventStream::from(&*TERM);
		let mut rx = stream.take().unwrap();

		let result = async {
			let emulator = Self::from_env();
			emulator.request()?;

			loop {
				let wait_da1 = async {
					while let Some(event) = rx.recv().await {
						let Event::Report(report) = event? else { continue };

						emulator.apply(&report);
						if matches!(report, Report::Da1(_)) {
							return Ok(());
						}
					}
					bail!("Terminal event stream closed during emulator detection");
				};

				match time::timeout(Duration::from_secs(3), wait_da1).await {
					Ok(result) => result?,
					Err(_) => return Ok(emulator),
				}

				if !emulator.needs_passthrough() {
					return Ok(emulator);
				}

				Mux::tmux_setup().await;
				if let Err(e) = emulator.restart() {
					error!("Failed to request terminal capabilities through tmux: {e}");
					return Ok(emulator);
				}
			}
		}
		.await;

		drop(rx);
		drop(stream);
		TERM.enter_cooked_mode()?;
		result
	}

	pub fn needs_passthrough(&self) -> bool {
		self.brand.get() == Brand::Tmux && self.mux.get().is_none()
	}

	fn request(&self) -> Result<()> {
		let w = |t: &'static dyn Display| TmuxPassthrough(t, self.mux.get().is_some());

		writef!(
			TTY.writer(),
			"{SaveCursorPos}{RequestColorScheme}{RequestBgColor}{RequestCursorBlink}{RequestCursorStyle}{}{}{RequestCellPixelSize}{ProbeClipboard}{RequestCsiU}{}{RestoreCursorPos}",
			w(&RequestXtVersion),
			w(&RequestKittyGraphics),
			w(&RequestDA1),
		)?;

		Ok(())
	}
}
