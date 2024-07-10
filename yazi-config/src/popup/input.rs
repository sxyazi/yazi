use std::str::FromStr;

use serde::Deserialize;

use super::{Offset, Origin};

#[derive(Deserialize)]
pub struct Input {
	pub cursor_blink: bool,

	// cd
	pub cd_title:  String,
	pub cd_origin: Origin,
	pub cd_offset: Offset,

	// create
	pub create_title:  String,
	pub create_origin: Origin,
	pub create_offset: Offset,

	// rename
	pub rename_title:  String,
	pub rename_origin: Origin,
	pub rename_offset: Offset,

	// trash
	pub trash_title:  String,
	pub trash_origin: Origin,
	pub trash_offset: Offset,

	// delete
	pub delete_title:  String,
	pub delete_origin: Origin,
	pub delete_offset: Offset,

	// filter
	pub filter_title:  String,
	pub filter_origin: Origin,
	pub filter_offset: Offset,

	// find
	pub find_title:  [String; 2],
	pub find_origin: Origin,
	pub find_offset: Offset,

	// search
	pub search_title:  String,
	pub search_origin: Origin,
	pub search_offset: Offset,

	// shell
	pub shell_title:  [String; 2],
	pub shell_origin: Origin,
	pub shell_offset: Offset,

	// overwrite
	pub overwrite_title:  String,
	pub overwrite_origin: Origin,
	pub overwrite_offset: Offset,

	// quit
	pub quit_title:  String,
	pub quit_origin: Origin,
	pub quit_offset: Offset,
}

impl Input {
	pub const fn border(&self) -> u16 { 2 }
}

impl FromStr for Input {
	type Err = toml::de::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		#[derive(Deserialize)]
		struct Outer {
			input: Input,
		}

		Ok(toml::from_str::<Outer>(s)?.input)
	}
}
