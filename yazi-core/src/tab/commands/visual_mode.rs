use std::collections::BTreeSet;

use yazi_macro::render;
use yazi_parser::tab::VisualModeOpt;

use crate::tab::{Mode, Tab};

impl Tab {
	#[yazi_codegen::command]
	pub fn visual_mode(&mut self, opt: VisualModeOpt) {
		let idx = self.current.cursor;
		if opt.unset {
			self.mode = Mode::Unset(idx, BTreeSet::from([idx]));
		} else {
			self.mode = Mode::Select(idx, BTreeSet::from([idx]));
		};
		render!();
	}
}
