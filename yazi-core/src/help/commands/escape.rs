use yazi_shared::event::Exec;

use crate::help::Help;

impl Help {
	pub fn escape(&mut self, _: &Exec) -> bool {
		if self.in_filter.is_some() {
			self.in_filter = None;
			self.filter_apply();
			true
		} else {
			self.toggle(self.layer)
		}
	}
}
