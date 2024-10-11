use yazi_shared::{event::{Cmd, Data}, render};

use crate::manager::Tabs;

pub struct Opt {
	step: isize,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { step: c.first().and_then(Data::as_isize).unwrap_or(0) } }
}

impl Tabs {
	#[yazi_codegen::command]
	pub fn swap(&mut self, opt: Opt) {
		let idx = self.absolute(opt.step);
		if idx == self.cursor {
			return;
		}

		self.items.swap(self.cursor, idx);
		self.set_idx(idx);
		self.reorder();
		render!();
	}
}
