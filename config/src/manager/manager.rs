use serde::{Deserialize, Serialize};

use super::{ManagerLayout, SortBy};
use crate::MERGED_YAZI;

#[derive(Debug, Deserialize, Serialize)]
pub struct Manager {
	pub layout: ManagerLayout,

	// Sorting
	pub sort_by:        SortBy,
	pub sort_sensitive: bool,
	pub sort_reverse:   bool,
	pub sort_dir_first: bool,

	// Display
	pub show_hidden:  bool,
	pub show_symlink: bool,
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
