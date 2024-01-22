use std::str::FromStr;

use yazi_config::manager::SortBy;
use yazi_shared::event::Exec;

use crate::{manager::Manager, tab::Tab, tasks::Tasks};

impl Tab {
	pub fn sort(&mut self, e: Exec, tasks: &Tasks) {
		if let Some(by) = e.args.first() {
			self.conf.sort_by = SortBy::from_str(by).unwrap_or_default();
		}
		self.conf.sort_sensitive = e.named.contains_key("sensitive");
		self.conf.sort_reverse = e.named.contains_key("reverse");
		self.conf.sort_dir_first = e.named.contains_key("dir-first");

		self.apply_files_attrs();
		Manager::_update_paged();

		tasks.preload_sorted(&self.current.files);
	}
}
