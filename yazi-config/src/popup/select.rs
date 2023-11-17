use serde::Deserialize;

use super::{Offset, Origin};
use crate::MERGED_YAZI;

#[derive(Deserialize)]
pub struct Select {
	// open
	pub open_title:  String,
	pub open_origin: Origin,
	pub open_offset: Offset,
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

impl Select {
	#[inline]
	pub const fn border(&self) -> u16 { 2 }
}
