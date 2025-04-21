use yazi_proxy::MgrProxy;
use yazi_shared::event::CmdCow;

use crate::tab::Tab;

impl Tab {
	pub fn hidden(&mut self, mut c: CmdCow) {
		self.pref.show_hidden = match c.take_first_str().as_deref() {
			Some("show") => true,
			Some("hide") => false,
			_ => !self.pref.show_hidden,
		};

		let hovered = self.hovered().map(|f| f.url_owned());
		self.apply_files_attrs();

		if hovered.as_ref() != self.hovered().map(|f| &f.url) {
			self.hover(hovered);
			MgrProxy::peek(false);
			MgrProxy::watch();
		} else if self.hovered().is_some_and(|f| f.is_dir()) {
			MgrProxy::peek(true);
		}
		MgrProxy::update_paged();
	}
}
