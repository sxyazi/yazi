use yazi_shared::event::CmdCow;

use crate::tab::Tab;

impl Tab {
	pub fn forward(&mut self, _: CmdCow) {
		self.backstack.shift_forward().cloned().map(|u| self.cd(u));
	}
}
