use yazi_term::{TERM, event::{Event, Report}};

use crate::EMULATOR;

#[must_use]
pub struct Deinit;

impl Drop for Deinit {
	fn drop(&mut self) {
		if let Some(dur) = EMULATOR.probe.remaining() {
			_ = TERM.source.try_poll(Some(dur), |event| matches!(event, Event::Report(Report::Da1(_))));
		}

		TERM.source.drain().ok();
		EMULATOR.stop();
	}
}
