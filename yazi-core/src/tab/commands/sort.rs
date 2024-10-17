use std::str::FromStr;

use yazi_config::manager::SortBy;
use yazi_proxy::ManagerProxy;
use yazi_shared::event::Cmd;

use crate::{tab::Tab, tasks::Tasks};

impl Tab {
	pub fn sort(&mut self, mut c: Cmd, tasks: &Tasks) {
		let pref = &mut self.pref;
		if let Some(by) = c.take_first_str() {
			pref.sort_by = SortBy::from_str(&by).unwrap_or_default();
		}

		pref.sort_reverse = c.maybe_bool("reverse").unwrap_or(pref.sort_reverse);
		pref.sort_dir_first = c.maybe_bool("dir-first").unwrap_or(pref.sort_dir_first);
		pref.sort_sensitive = c.maybe_bool("sensitive").unwrap_or(pref.sort_sensitive);
		pref.sort_translit = c.maybe_bool("translit").unwrap_or(pref.sort_translit);

		self.apply_files_attrs();
		ManagerProxy::update_paged();

		tasks.prework_sorted(&self.current.files);
	}
}
