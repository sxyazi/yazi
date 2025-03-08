use yazi_macro::render;
use yazi_shared::event::{CmdCow, Data};

use crate::mgr::Tabs;

struct Opt {
	step:     isize,
	relative: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self { step: c.first().and_then(Data::as_isize).unwrap_or(0), relative: c.bool("relative") }
	}
}

impl Tabs {
	#[yazi_codegen::command]
	pub fn switch(&mut self, opt: Opt) {
		let idx = if opt.relative {
			opt.step.saturating_add_unsigned(self.cursor).rem_euclid(self.items.len() as _) as _
		} else {
			opt.step as usize
		};

		if idx == self.cursor || idx >= self.items.len() {
			return;
		}

		self.set_idx(idx);
		render!();
	}
}
