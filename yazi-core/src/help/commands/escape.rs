use yazi_config::keymap::Exec;

use crate::help::Help;

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Help {
	pub fn escape(&mut self, _: impl Into<Opt>) -> bool {
		if self.in_filter.is_some() {
			self.in_filter = None;
			self.filter_apply();
			true
		} else {
			self.toggle(self.layer)
		}
	}
}
