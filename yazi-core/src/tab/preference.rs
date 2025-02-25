use yazi_config::MGR;
use yazi_fs::{FilesSorter, SortBy};

#[derive(Clone, PartialEq)]
pub struct Preference {
	// Sorting
	pub sort_by:        SortBy,
	pub sort_sensitive: bool,
	pub sort_reverse:   bool,
	pub sort_dir_first: bool,
	pub sort_translit:  bool,

	// Display
	pub linemode:    String,
	pub show_hidden: bool,
}

impl Default for Preference {
	fn default() -> Self {
		Self {
			// Sorting
			sort_by:        MGR.sort_by,
			sort_sensitive: MGR.sort_sensitive,
			sort_reverse:   MGR.sort_reverse,
			sort_dir_first: MGR.sort_dir_first,
			sort_translit:  MGR.sort_translit,

			// Display
			linemode:    MGR.linemode.to_owned(),
			show_hidden: MGR.show_hidden,
		}
	}
}

impl Preference {
	pub(super) fn patch<F: FnOnce(&mut Self)>(&mut self, f: F) -> bool {
		let old = self.clone();
		f(self);
		*self != old
	}
}

impl From<&Preference> for FilesSorter {
	fn from(value: &Preference) -> Self {
		FilesSorter {
			by:        value.sort_by,
			sensitive: value.sort_sensitive,
			reverse:   value.sort_reverse,
			dir_first: value.sort_dir_first,
			translit:  value.sort_translit,
		}
	}
}
