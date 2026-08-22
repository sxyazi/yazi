use std::{fmt::Display, time::Duration};

use anyhow::{Result, bail};
use tokio::{sync::Notify, time::{self, timeout}};
use yazi_macro::{error, writef};
use yazi_shared::id::{Id, Ids};
use yazi_shim::cell::SyncCell;
use yazi_term::{TERM, event::{Event, Report}, stream::EventStream};
use yazi_tty::{TTY, sequence::{ProbeClipboard, RequestBgColor, RequestCellPixelSize, RequestColorScheme, RequestCsiU, RequestCursorBlink, RequestCursorStyle, RequestDA1, RequestKittyGraphics, RequestXtVersion, RestoreCursorPos, SaveCursorPos}};

use crate::{Emulator, Mux};

static IDS: Ids = Ids::new();

#[derive(Debug, Default)]
pub struct Probe {
	pub id:    SyncCell<Id>,
	completed: SyncCell<bool>,
	notifier:  Notify,
}

impl Probe {
	pub(crate) fn reset(&self) {
		self.id.set(IDS.next());
		self.completed.set(false);
	}

	pub fn complete(&self) {
		if !self.completed.replace(true) {
			self.notifier.notify_waiters();
		}
	}

	pub fn pending(&self) -> Option<Id> { (!self.completed.get()).then(|| self.id.get()) }

	pub async fn wait(&self, id: Id) {
		loop {
			if self.completed.get() {
				return;
			}

			let notified = self.notifier.notified();
			if self.completed.get() {
				return;
			} else if timeout(Duration::from_secs(5), notified).await.is_err() {
				self.cancel(id);
			}
		}
	}

	pub fn cancel(&self, id: Id) {
		if self.id == id {
			self.complete();
		}
	}
}

impl Emulator {
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

	pub(super) fn request(&self) -> Result<()> {
		let w = |t: &'static dyn Display| TTY.tmux_passthrough(t);

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
