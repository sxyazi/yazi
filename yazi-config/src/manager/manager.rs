use std::str::FromStr;

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::{ManagerRatio, MouseEvents, SortBy};

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
	pub mouse_events: MouseEvents,
	pub title_format: String,
}

impl FromStr for Manager {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		#[derive(Deserialize)]
		struct Outer {
			manager: Manager,
		}

		let manager = toml::from_str::<Outer>(s)?.manager;
		manager.validate()?;

		Ok(manager)
	}
}
