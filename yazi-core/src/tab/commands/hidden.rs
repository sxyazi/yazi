use yazi_proxy::ManagerProxy;
use yazi_shared::event::Cmd;

use crate::tab::Tab;

impl Tab {
	pub fn hidden(&mut self, mut c: Cmd) {
		self.conf.show_hidden = match c.take_first_str().as_deref() {
			Some("show") => true,
			Some("hide") => false,
			_ => !self.conf.show_hidden,
		};

		let hovered = self.current.hovered().map(|f| f.url_owned());
		self.apply_files_attrs();

		if hovered.as_ref() != self.current.hovered().map(|f| f.url()) {
			ManagerProxy::hover(hovered, self.idx);
		} else if self.current.hovered().is_some_and(|f| f.is_dir()) {
			ManagerProxy::peek(true);
		}
		ManagerProxy::update_paged();
	}
}
