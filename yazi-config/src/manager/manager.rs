use serde::{Deserialize, Serialize};
use validator::Validate;

use super::{ManagerRatio, SortBy};
use crate::{validation::check_validation, MERGED_YAZI};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Manager {
	pub ratio: ManagerRatio,

	// Sorting
	pub sort_by:        SortBy,
	pub sort_sensitive: bool,
	pub sort_reverse:   bool,
	pub sort_dir_first: bool,
	pub sort_translit:  bool,

	// Display
	#[validate(length(min = 1, max = 20, message = "must be between 1 and 20 characters"))]
	pub linemode:     String,
	pub show_hidden:  bool,
	pub show_symlink: bool,
	pub scrolloff:    u8,
}

impl Default for Manager {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			manager: Manager,
		}

		let manager = toml::from_str::<Outer>(&MERGED_YAZI).unwrap().manager;

		check_validation(manager.validate());
		manager
	}
}
