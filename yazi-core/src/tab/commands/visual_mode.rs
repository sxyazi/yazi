use std::collections::BTreeSet;

use yazi_shared::{event::Cmd, render};

use crate::tab::{Mode, Tab};

pub struct Opt {
	unset: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { unset: c.bool("unset") } }
}

impl Tab {
	#[yazi_codegen::command]
	pub fn visual_mode(&mut self, opt: Opt) {
		let idx = self.current.cursor;
		if opt.unset {
			self.mode = Mode::Unset(idx, BTreeSet::from([idx]));
		} else {
			self.mode = Mode::Select(idx, BTreeSet::from([idx]));
		};
		render!();
	}
}
