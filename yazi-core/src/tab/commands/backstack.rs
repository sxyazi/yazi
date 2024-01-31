use yazi_shared::event::Cmd;

use crate::tab::Tab;

pub struct Opt;
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}
impl From<Cmd> for Opt {
	fn from(_: Cmd) -> Self { Self }
}

impl Tab {
	pub fn back(&mut self, _: impl Into<Opt>) {
		if let Some(url) = self.backstack.shift_backward().cloned() {
			self.cd(url);
		}
	}

	pub fn forward(&mut self, _: impl Into<Opt>) {
		if let Some(url) = self.backstack.shift_forward().cloned() {
			self.cd(url);
		}
	}
}
