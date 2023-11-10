use yazi_config::keymap::Exec;

use crate::help::Help;

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Help {
	pub fn filter(&mut self, _: impl Into<Opt>) -> bool {
		self.in_filter = Some(Default::default());
		self.filter_apply();
		true
	}
}
