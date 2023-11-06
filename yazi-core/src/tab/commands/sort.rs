use std::str::FromStr;

use yazi_config::{keymap::Exec, manager::SortBy};

use crate::tab::Tab;

impl Tab {
	pub fn sort(&mut self, e: &Exec) -> bool {
		if let Some(by) = e.args.first() {
			self.conf.sort_by = SortBy::from_str(by).unwrap_or_default();
		}
		self.conf.sort_sensitive = e.named.contains_key("sensitive");
		self.conf.sort_reverse = e.named.contains_key("reverse");
		self.conf.sort_dir_first = e.named.contains_key("dir_first");

		self.apply_files_attrs(false)
	}
}
