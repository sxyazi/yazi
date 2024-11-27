use yazi_shared::event::CmdCow;

use crate::tab::Tab;

impl Tab {
	pub fn back(&mut self, _: CmdCow) {
		self.backstack.shift_backward().cloned().map(|u| self.cd(u));
	}
}
