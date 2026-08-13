use std::{fmt::Display, time::Duration};

use anyhow::{Result, bail};
use tokio::time;
use yazi_macro::{error, writef};
use yazi_shared::id::{Id, Ids};
use yazi_term::{TERM, event::{Event, Report}, stream::EventStream};
use yazi_tty::{TTY, sequence::{ProbeClipboard, RequestBgColor, RequestCellPixelSize, RequestColorScheme, RequestCsiU, RequestCursorBlink, RequestCursorStyle, RequestDA1, RequestKittyGraphics, RequestXtVersion, RestoreCursorPos, SaveCursorPos, TmuxPassthrough}};

use crate::{Brand, Emulator, Mux};

static IDS: Ids = Ids::new();

pub struct Probe {
	pub id:       Id,
	pub emulator: Emulator,
}

impl Probe {
	pub fn start() -> Result<Self> {
		let probe = Self { id: IDS.next(), emulator: Emulator::from_env() };
		probe.request()?;
		Ok(probe)
	}

	pub fn restart(&mut self) -> Result<()> {
		self.id = IDS.next();
		self.emulator =
			Emulator { mux: Some(Mux { sixel: self.emulator.sixel }), ..Default::default() };
		self.request()
	}

	pub fn needs_passthrough(&self) -> bool {
		self.emulator.brand == Brand::Tmux && self.emulator.mux.is_none()
	}

	fn request(&self) -> Result<()> {
		let w = |t: &'static dyn Display| TmuxPassthrough(t, self.emulator.mux.is_some());

		writef!(
			TTY.writer(),
			"{SaveCursorPos}{}{RequestCursorBlink}{RequestCursorStyle}{RequestColorScheme}{RequestBgColor}{}{RequestCellPixelSize}{ProbeClipboard}{RequestCsiU}{}{RestoreCursorPos}",
			w(&RequestXtVersion),
			w(&RequestKittyGraphics),
			w(&RequestDA1),
		)?;

		Ok(())
	}
}

impl Emulator {
	pub async fn probe() -> Result<Self> {
		TERM.enter_raw_mode()?;
		let mut stream = EventStream::from(&*TERM);
		let mut rx = stream.take().unwrap();

		let result = async {
			let mut probe = Probe::start()?;
			loop {
				let wait_da1 = async {
					while let Some(event) = rx.recv().await {
						let Event::Report(report) = event? else { continue };

						probe.emulator.apply(&report);
						if matches!(report, Report::Da1(_)) {
							return Ok(());
						}
					}
					bail!("Terminal event stream closed during emulator detection");
				};

				match time::timeout(Duration::from_secs(3), wait_da1).await {
					Ok(result) => result?,
					Err(_) => return Ok(probe.emulator),
				}

				if !probe.needs_passthrough() {
					return Ok(probe.emulator);
				}

				Mux::tmux_setup().await;
				if let Err(e) = probe.restart() {
					error!("Failed to request terminal capabilities through tmux: {e}");
					return Ok(probe.emulator);
				}
			}
		}
		.await;

		drop(rx);
		drop(stream);
		TERM.enter_cooked_mode()?;
		result
	}
}
