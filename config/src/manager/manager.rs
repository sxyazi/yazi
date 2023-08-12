use serde::Deserialize;

use super::SortBy;
use crate::MERGED_YAZI;

#[derive(Debug, Deserialize)]
pub struct Manager {
	// Sorting
	pub sort_by:      SortBy,
	pub sort_reverse: bool,
	pub sort_dir_first: bool,

	// Display
	pub show_hidden: bool,
}

impl Default for Manager {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			manager: Manager,
		}

		toml::from_str::<Outer>(&MERGED_YAZI).unwrap().manager
	}
}
