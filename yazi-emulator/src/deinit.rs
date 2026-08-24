use std::time::Duration;

use yazi_term::{TERM, event::{Event, Report}};

use crate::EMULATOR;

#[must_use]
pub struct Deinit;

impl Drop for Deinit {
	fn drop(&mut self) {
		if EMULATOR.probe.pending().is_some() {
			_ = TERM.source.try_poll(Some(Duration::from_secs(3)), |event| {
				matches!(event, Event::Report(Report::Da1(_)))
			});
		}

		TERM.source.drain().ok();
		EMULATOR.stop();
	}
}
