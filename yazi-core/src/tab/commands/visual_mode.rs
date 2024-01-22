use std::collections::BTreeSet;

use yazi_shared::{event::Exec, render};

use crate::tab::{Mode, Tab};

pub struct Opt {
	unset: bool,
}

impl From<Exec> for Opt {
	fn from(e: Exec) -> Self { Self { unset: e.named.contains_key("unset") } }
}

impl Tab {
	pub fn visual_mode(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		let idx = self.current.cursor;

		if opt.unset {
			self.mode = Mode::Unset(idx, BTreeSet::from([idx]));
		} else {
			self.mode = Mode::Select(idx, BTreeSet::from([idx]));
		};
		render!();
	}
}
