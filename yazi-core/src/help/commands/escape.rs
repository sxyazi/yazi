use yazi_shared::{event::Cmd, render};

use crate::help::Help;

impl Help {
	pub fn escape(&mut self, _: Cmd) {
		if self.keyword().is_none() {
			return self.toggle(self.layer);
		}

		self.keyword = String::new();
		self.in_filter = None;
		self.filter_apply();
		render!();
	}
}
