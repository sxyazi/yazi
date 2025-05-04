use yazi_shared::event::CmdCow;

use crate::tab::Tab;

impl Tab {
	pub fn back(&mut self, _: CmdCow) {
		if self.current.url.is_regular() {
			self.backstack.push(&self.current.url);
		}
		if let Some(u) = self.backstack.shift_backward().cloned() {
			self.cd((u, super::cd::OptSource::Back));
		}
	}
}
