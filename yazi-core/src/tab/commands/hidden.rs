use yazi_shared::event::Exec;

use crate::{manager::Manager, tab::Tab};

impl Tab {
	pub fn hidden(&mut self, e: &Exec) {
		self.conf.show_hidden = match e.args.first().map(|s| s.as_bytes()) {
			Some(b"show") => true,
			Some(b"hide") => false,
			_ => !self.conf.show_hidden,
		};

		let hovered = self.current.hovered().map(|f| f.url());
		self.apply_files_attrs();

		if hovered.as_ref() != self.current.hovered().map(|f| &f.url) {
			Manager::_hover(hovered);
		} else if self.current.hovered().is_some_and(|f| f.is_dir()) {
			Manager::_peek(true);
		}
		Manager::_update_paged();
	}
}
