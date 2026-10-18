use std::{fmt, time::Duration};

use anyhow::{Result, bail};
use tokio::{sync::Notify, time::{self, Instant, timeout_at}};
use yazi_macro::{error, writef};
use yazi_shared::id::{Id, Ids};
use yazi_shim::cell::SyncCell;
use yazi_term::{TERM, event::Event, stream::EventStream};
use yazi_tty::{TTY, sequence::{EraseLine, ProbeClipboard, RequestBgColor, RequestCellPixelSize, RequestColorScheme, RequestCsiU, RequestCursorBlink, RequestCursorStyle, RequestDA1, RequestKgp, RequestKgpShm, RequestXtVersion, RestoreCursor, SaveCursor, TmuxPassthrough}};

use crate::{Emulator, Mux};

static IDS: Ids = Ids::new();

#[derive(Default)]
pub struct Probe {
	pub id:   SyncCell<Id>,
	deadline: SyncCell<Option<Instant>>,
	notifier: Notify,
}

impl fmt::Debug for Probe {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Probe").field("id", &self.id).field("deadline", &self.deadline).finish()
	}
}

impl Probe {
	pub(crate) fn reset(&self) {
		self.id.set(IDS.next());
		self.deadline.set(Some(Instant::now() + Duration::from_secs(2)));
		self.notifier.notify_waiters();
	}

	pub fn defer(&self) { self.deadline.set(Some(Instant::now() + Duration::from_secs(7))); }

	pub fn complete(&self) {
		self.deadline.set(None);
		self.notifier.notify_waiters();
	}

	pub fn pending(&self) -> Option<Instant> { self.deadline.get().filter(|&d| d > Instant::now()) }

	pub fn remaining(&self) -> Option<Duration> {
		self.deadline.get()?.checked_duration_since(Instant::now()).filter(|d| !d.is_zero())
	}

	pub async fn wait(&self) {
		loop {
			let notified = self.notifier.notified();
			let Some(deadline) = self.pending() else { return };
			_ = timeout_at(deadline, notified).await;
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
						if report.is_da_1() {
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
		let mux = self.mux.get().is_some();
		writef!(
			TTY.writer(),
			"{SaveCursor}{RequestColorScheme}{RequestBgColor}{RequestCursorBlink}{RequestCursorStyle}{}{}{}{RequestCellPixelSize}{ProbeClipboard}{RequestCsiU}{}{}{RestoreCursor}",
			TmuxPassthrough(RequestXtVersion, mux),
			TmuxPassthrough(RequestKgp, mux),
			TmuxPassthrough(RequestKgpShm::new(), mux),
			TmuxPassthrough(RequestDA1, mux),
			EraseLine::BeforeCursor
		)?;

		Ok(())
	}
}
