use std::collections::BTreeSet;

use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::tab::{Mode, Tab};

struct Opt {
	unset: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { unset: c.bool("unset") } }
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
