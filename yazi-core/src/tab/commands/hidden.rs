use yazi_proxy::ManagerProxy;
use yazi_shared::event::Cmd;

use crate::tab::Tab;

impl Tab {
	pub fn hidden(&mut self, c: Cmd) {
		self.conf.show_hidden = match c.args.first().map(|s| s.as_str()) {
			Some("show") => true,
			Some("hide") => false,
			_ => !self.conf.show_hidden,
		};

		let hovered = self.current.hovered().map(|f| f.url());
		self.apply_files_attrs();

		if hovered.as_ref() != self.current.hovered().map(|f| &f.url) {
			ManagerProxy::hover(hovered);
		} else if self.current.hovered().is_some_and(|f| f.is_dir()) {
			ManagerProxy::peek(true);
		}
		ManagerProxy::update_paged();
	}
}
