use yazi_shared::event::CmdCow;

use crate::tab::Tab;

impl Tab {
	pub fn enter(&mut self, _: CmdCow) {
		self
			.hovered()
			.filter(|h| h.is_dir())
			.map(|h| h.url.to_regular())
			.map(|u| self.cd((u, super::cd::OptSource::Enter)));
	}
}
