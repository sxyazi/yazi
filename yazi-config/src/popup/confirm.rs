use std::str::FromStr;

use anyhow::Context;
use serde::Deserialize;

use super::{Offset, Origin};

#[derive(Deserialize)]
pub struct Confirm {
	// trash
	pub trash_title:  String,
	pub trash_origin: Origin,
	pub trash_offset: Offset,

	// delete
	pub delete_title:  String,
	pub delete_origin: Origin,
	pub delete_offset: Offset,

	// overwrite
	pub overwrite_title:   String,
	pub overwrite_content: String,
	pub overwrite_origin:  Origin,
	pub overwrite_offset:  Offset,

	// quit
	pub quit_title:   String,
	pub quit_content: String,
	pub quit_origin:  Origin,
	pub quit_offset:  Offset,
}

impl FromStr for Confirm {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		#[derive(Deserialize)]
		struct Outer {
			confirm: Confirm,
		}

		let outer = toml::from_str::<Outer>(s)
			.context("Failed to parse the [confirm] section in your yazi.toml")?;

		Ok(outer.confirm)
	}
}

impl Confirm {
	pub const fn border(&self) -> u16 { 2 }
}
