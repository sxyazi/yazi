use serde::Deserialize;

use super::position::{Offset, Position};
use crate::MERGED_YAZI;

#[derive(Debug, Deserialize)]
pub struct Input {
	// cd
	pub cd_position:     Position,
	pub cd_offset:       Offset,
	// search
	pub search_position: Position,
	pub search_offset:   Offset,
	// find
	pub find_position:   Position,
	pub find_offset:     Offset,
	// shell
	pub shell_position:  Position,
	pub shell_offset:    Offset,
	// create
	pub create_position: Position,
	pub create_offset:   Offset,
	// rename
	pub rename_position: Position,
	pub rename_offset:   Offset,
}

impl Default for Input {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			input: Input,
		}

		// TODO:
		toml::from_str::<Outer>(&MERGED_YAZI).unwrap().input
	}
}
