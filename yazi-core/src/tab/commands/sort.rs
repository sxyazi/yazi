use std::str::FromStr;

use yazi_config::manager::SortBy;
use yazi_proxy::ManagerProxy;
use yazi_shared::event::Cmd;

use crate::{tab::Tab, tasks::Tasks};

impl Tab {
	pub fn sort(&mut self, mut c: Cmd, tasks: &Tasks) {
		if let Some(by) = c.take_first_str() {
			self.conf.sort_by = SortBy::from_str(&by).unwrap_or_default();
		}
		self.conf.sort_sensitive = c.get_bool("sensitive");
		self.conf.sort_reverse = c.get_bool("reverse");
		self.conf.sort_dir_first = c.get_bool("dir-first");

		self.apply_files_attrs();
		ManagerProxy::update_paged();

		tasks.preload_sorted(&self.current.files);
	}
}
