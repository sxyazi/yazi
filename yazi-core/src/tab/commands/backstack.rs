use yazi_shared::event::Exec;

use crate::tab::Tab;

pub struct Opt;
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}
impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Tab {
	pub fn back(&mut self, _: impl Into<Opt>) -> bool {
		if let Some(url) = self.backstack.shift_backward().cloned() {
			self.cd(url);
		}
		false
	}

	pub fn forward(&mut self, _: impl Into<Opt>) -> bool {
		if let Some(url) = self.backstack.shift_forward().cloned() {
			self.cd(url);
		}
		false
	}
}
