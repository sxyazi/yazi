use yazi_parser::tab::CdSource;
use yazi_shared::event::CmdCow;

use crate::tab::Tab;

impl Tab {
	pub fn forward(&mut self, _: CmdCow) {
		if let Some(u) = self.backstack.shift_forward().cloned() {
			self.cd((u, CdSource::Forward));
		}
	}
}
