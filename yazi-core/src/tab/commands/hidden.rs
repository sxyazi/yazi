use yazi_shared::{event::Exec, render};

use crate::{manager::Manager, tab::Tab};

impl Tab {
	pub fn hidden(&mut self, e: &Exec) {
		self.conf.show_hidden = match e.args.first().map(|s| s.as_bytes()) {
			Some(b"show") => true,
			Some(b"hide") => false,
			_ => !self.conf.show_hidden,
		};
		if self.apply_files_attrs(false) {
			Manager::_hover(None);
			render!();
		}
	}
}
