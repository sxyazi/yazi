use yazi_shared::event::CmdCow;

use super::cd::CdSource;
use crate::tab::Tab;

impl Tab {
	pub fn back(&mut self, _: CmdCow) {
		if let Some(u) = self.backstack.shift_backward().cloned() {
			self.cd((u, CdSource::Back));
		}
	}
}
