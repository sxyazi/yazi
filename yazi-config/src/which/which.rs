use std::str::FromStr;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::SortBy;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Which {
	// Sorting
	pub sort_by:        SortBy,
	pub sort_sensitive: bool,
	pub sort_reverse:   bool,
	pub sort_translit:  bool,
}

impl FromStr for Which {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		#[derive(Deserialize)]
		struct Outer {
			which: Which,
		}

		let outer: Outer =
			toml::from_str(s).context("Failed to parse the [which] section in your yazi.toml")?;

		Ok(outer.which)
	}
}
