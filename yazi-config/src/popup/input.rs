use serde::Deserialize;

use super::{Offset, Origin};
use crate::MERGED_YAZI;

#[derive(Deserialize)]
pub struct Input {
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
}

impl Default for Input {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			input: Input,
		}

		toml::from_str::<Outer>(&MERGED_YAZI).unwrap().input
	}
}
