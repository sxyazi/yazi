use yazi_shared::{event::Exec, render};

use crate::help::Help;

impl Help {
	pub fn escape(&mut self, _: Exec) {
		if self.in_filter.is_none() {
			return self.toggle(self.layer);
		}

		self.in_filter = None;
		self.filter_apply();
		render!();
	}
}
