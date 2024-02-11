use serde::{Deserialize, Serialize};
use validator::Validate;

use super::SortBy;
use crate::{validation::check_validation, MERGED_YAZI};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Which {
	// Sorting
	pub sort_by:        SortBy,
	pub sort_sensitive: bool,
	pub sort_reverse:   bool,
}

impl Default for Which {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			which: Which,
		}

		let which = toml::from_str::<Outer>(&MERGED_YAZI).unwrap().which;

		check_validation(which.validate());
		which
	}
}
