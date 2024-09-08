use yazi_shared::event::Cmd;

use crate::tab::Tab;

impl Tab {
	pub fn enter(&mut self, _: Cmd) {
		self.current.hovered().filter(|h| h.is_dir()).map(|h| h.url_owned()).map(|u| self.cd(u));
	}
}
