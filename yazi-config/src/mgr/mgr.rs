use std::str::FromStr;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use validator::Validate;
use yazi_fs::SortBy;

use super::{MgrRatio, MouseEvents};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Mgr {
	pub ratio: MgrRatio,

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

impl FromStr for Mgr {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		#[derive(Deserialize)]
		struct Outer {
			#[serde(rename = "manager")]
			mgr: Mgr, // TODO: remove serde(rename)
		}

		let outer = toml::from_str::<Outer>(s)
			.context("Failed to parse the [manager] section in your yazi.toml")?;
		outer.mgr.validate()?;

		Ok(outer.mgr)
	}
}
