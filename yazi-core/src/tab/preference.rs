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
			sort_by:        YAZI.mgr.sort_by.get(),
			sort_sensitive: YAZI.mgr.sort_sensitive.get(),
			sort_reverse:   YAZI.mgr.sort_reverse.get(),
			sort_dir_first: YAZI.mgr.sort_dir_first.get(),
			sort_translit:  YAZI.mgr.sort_translit.get(),

			// Display
			linemode:    YAZI.mgr.linemode.clone(),
			show_hidden: YAZI.mgr.show_hidden.get(),
		}
	}
}

impl From<&Preference> for FilesSorter {
	fn from(value: &Preference) -> Self {
		Self {
			by:        value.sort_by,
			sensitive: value.sort_sensitive,
			reverse:   value.sort_reverse,
			dir_first: value.sort_dir_first,
			translit:  value.sort_translit,
		}
	}
}
