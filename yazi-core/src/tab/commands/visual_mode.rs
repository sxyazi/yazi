use std::collections::BTreeSet;

use yazi_shared::{event::Cmd, render};

use crate::tab::{Mode, Tab};

pub struct Opt {
	unset: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { unset: c.named.contains_key("unset") } }
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
