use yazi_shared::event::Cmd;

use crate::tab::Tab;

impl Tab {
	pub fn back(&mut self, _: Cmd) { self.backstack.shift_backward().cloned().map(|u| self.cd(u)); }
}
