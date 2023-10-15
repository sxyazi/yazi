use std::collections::BTreeSet;

use crate::tab::{Mode, Tab};

impl Tab {
	pub fn visual_mode(&mut self, unset: bool) -> bool {
		let idx = self.current.cursor;

		if unset {
			self.mode = Mode::Unset(idx, BTreeSet::from([idx]));
		} else {
			self.mode = Mode::Select(idx, BTreeSet::from([idx]));
		};
		true
	}
}
