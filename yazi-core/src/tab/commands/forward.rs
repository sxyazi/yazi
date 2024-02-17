use yazi_shared::event::Cmd;

use crate::tab::Tab;

impl Tab {
	pub fn forward(&mut self, _: Cmd) { self.backstack.shift_forward().cloned().map(|u| self.cd(u)); }
}
