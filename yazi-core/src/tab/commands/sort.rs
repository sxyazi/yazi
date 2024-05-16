use std::str::FromStr;

use yazi_config::manager::SortBy;
use yazi_proxy::ManagerProxy;
use yazi_shared::event::Cmd;

use crate::{tab::{Config, Tab}, tasks::Tasks};

impl Tab {
	pub fn sort(&mut self, mut c: Cmd, tasks: &Tasks) {
		let defaults = Config::default();
		if let Some(by) = c.take_first_str() {
			self.conf.sort_by = SortBy::from_str(&by).unwrap_or_default();
		} else {
			self.conf.sort_by = defaults.sort_by;
		}

		if let Some(rev) = c.maybe_bool("reverse") {
			self.conf.sort_reverse = rev;
		} else {
			self.conf.sort_reverse = defaults.sort_reverse;
		}

		if let Some(d_f) = c.maybe_bool("dir-first") {
			self.conf.sort_dir_first = d_f;
		} else {
			self.conf.sort_dir_first = defaults.sort_dir_first;
		}

		if let Some(sen) = c.maybe_bool("sensitive") {
			self.conf.sort_sensitive = sen;
		} else {
			self.conf.sort_sensitive = defaults.sort_sensitive;
		}

		if let Some(tran) = c.maybe_bool("transliteration") {
			self.conf.sort_transliteration = tran;
		} else {
			self.conf.sort_transliteration = defaults.sort_transliteration;
		}

		self.apply_files_attrs();
		ManagerProxy::update_paged();

		tasks.prework_sorted(&self.current.files);
	}
}
