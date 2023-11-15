use serde::Deserialize;

use super::position::{Offset, Position};
use crate::MERGED_YAZI;

#[derive(Debug, Deserialize)]
pub struct Input {
	// cd
	pub cd_title:    String,
	pub cd_position: Position,
	pub cd_offset:   Offset,

	// create
	pub create_title:    String,
	pub create_position: Position,
	pub create_offset:   Offset,

	// rename
	pub rename_title:    String,
	pub rename_position: Position,
	pub rename_offset:   Offset,

	// trash
	pub trash_title:    String,
	pub trash_position: Position,
	pub trash_offset:   Offset,

	// delete
	pub delete_title:    String,
	pub delete_position: Position,
	pub delete_offset:   Offset,

	// find
	pub find_title:    [String; 2],
	pub find_position: Position,
	pub find_offset:   Offset,

	// search
	pub search_title:    String,
	pub search_position: Position,
	pub search_offset:   Offset,

	// shell
	pub shell_title:    [String; 2],
	pub shell_position: Position,
	pub shell_offset:   Offset,
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
