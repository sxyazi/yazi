use yazi_config::{which::SortBy, WHICH};

use crate::which::WhichSorter;

#[derive(Clone, PartialEq, Debug)]
pub struct Config {
	// Sorting
	pub sort_by:        SortBy,
	pub sort_sensitive: bool,
	pub sort_reverse:   bool,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			// Sorting
			sort_by:        WHICH.sort_by,
			sort_sensitive: WHICH.sort_sensitive,
			sort_reverse:   WHICH.sort_reverse,
		}
	}
}

impl Config {

	#[inline]
	pub(super) fn sorter(&self) -> WhichSorter {
		WhichSorter {
			by:        self.sort_by,
			sensitive: self.sort_sensitive,
			reverse:   self.sort_reverse,
		}
	}
}
