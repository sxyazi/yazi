use yazi_config::{manager::SortBy, MANAGER};

use crate::folder::FilesSorter;

#[derive(Clone, PartialEq)]
pub struct Config {
	// Sorting
	pub sort_by:        SortBy,
	pub sort_sensitive: bool,
	pub sort_reverse:   bool,
	pub sort_dir_first: bool,

	// Display
	pub linemode:    String,
	pub show_hidden: bool,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			// Sorting
			sort_by:        MANAGER.sort_by,
			sort_sensitive: MANAGER.sort_sensitive,
			sort_reverse:   MANAGER.sort_reverse,
			sort_dir_first: MANAGER.sort_dir_first,

			// Display
			linemode:    MANAGER.linemode.to_owned(),
			show_hidden: MANAGER.show_hidden,
		}
	}
}

impl Config {
	pub(super) fn patch<F: FnOnce(&mut Self)>(&mut self, f: F) -> bool {
		let old = self.clone();
		f(self);
		*self != old
	}

	#[inline]
	pub(super) fn sorter(&self) -> FilesSorter {
		FilesSorter {
			by:        self.sort_by,
			sensitive: self.sort_sensitive,
			reverse:   self.sort_reverse,
			dir_first: self.sort_dir_first,
		}
	}
}
