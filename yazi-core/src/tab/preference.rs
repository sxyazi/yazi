use yazi_config::YAZI;
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
			sort_by:        YAZI.mgr.sort_by,
			sort_sensitive: YAZI.mgr.sort_sensitive,
			sort_reverse:   YAZI.mgr.sort_reverse,
			sort_dir_first: YAZI.mgr.sort_dir_first,
			sort_translit:  YAZI.mgr.sort_translit,

			// Display
			linemode:    YAZI.mgr.linemode.to_owned(),
			show_hidden: YAZI.mgr.show_hidden,
		}
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
