use config::keymap::Exec;

use crate::{emit, tab::Tab};

impl Tab {
	pub fn hidden(&mut self, e: &Exec) -> bool {
		self.conf.show_hidden = match e.args.get(0).map(|s| s.as_bytes()) {
			Some(b"show") => true,
			Some(b"hide") => false,
			_ => !self.conf.show_hidden,
		};
		if self.apply_files_attrs(false) {
			emit!(Peek);
			return true;
		}
		false
	}
}
