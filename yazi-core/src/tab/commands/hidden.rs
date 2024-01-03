use yazi_shared::event::Exec;

use crate::{manager::Manager, tab::Tab};

impl Tab {
	pub fn hidden(&mut self, e: &Exec) {
		self.conf.show_hidden = match e.args.first().map(|s| s.as_bytes()) {
			Some(b"show") => true,
			Some(b"hide") => false,
			_ => !self.conf.show_hidden,
		};

		self.apply_files_attrs();
		Manager::_hover(None);
	}
}
