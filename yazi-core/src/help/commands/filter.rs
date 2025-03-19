use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::help::Help;

impl Help {
	pub fn filter(&mut self, _: CmdCow) {
		self.in_filter = Some(Default::default());
		self.filter_apply();
		render!();
	}
}
