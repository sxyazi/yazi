use serde::Deserialize;

use super::{Offset, Origin};
use crate::MERGED_YAZI;

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
}

impl Default for Confirm {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			confirm: Confirm,
		}

		toml::from_str::<Outer>(&MERGED_YAZI).unwrap().confirm
	}
}

impl Confirm {
	#[inline]
	pub const fn border(&self) -> u16 { 2 }
}
