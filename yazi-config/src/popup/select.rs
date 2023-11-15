use serde::Deserialize;

use super::position::{Offset, Position};
use crate::MERGED_YAZI;

#[derive(Debug, Deserialize)]
pub struct Select {
	// open
	pub open_title:    String,
	pub open_position: Position,
	pub open_offset:   Offset,
}

impl Default for Select {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			select: Select,
		}

		toml::from_str::<Outer>(&MERGED_YAZI).unwrap().select
	}
}
