use std::str::FromStr;

use yazi_fs::SortBy;
use yazi_proxy::MgrProxy;
use yazi_shared::event::CmdCow;

use crate::{tab::Tab, tasks::Tasks};

impl Tab {
	pub fn sort(&mut self, c: CmdCow, tasks: &Tasks) {
		let mut new = self.pref.clone();
		new.sort_by = c.first_str().and_then(|s| SortBy::from_str(s).ok()).unwrap_or(new.sort_by);
		new.sort_reverse = c.maybe_bool("reverse").unwrap_or(new.sort_reverse);
		new.sort_dir_first = c.maybe_bool("dir-first").unwrap_or(new.sort_dir_first);
		new.sort_sensitive = c.maybe_bool("sensitive").unwrap_or(new.sort_sensitive);
		new.sort_translit = c.maybe_bool("translit").unwrap_or(new.sort_translit);

		if new == self.pref {
			return;
		}

		self.pref = new;
		self.apply_files_attrs();
		self.hover(None);
		tasks.prework_sorted(&self.current.files);

		MgrProxy::peek(false);
		MgrProxy::watch();
		MgrProxy::update_paged();
	}
}
