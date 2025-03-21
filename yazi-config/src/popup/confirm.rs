use serde::Deserialize;
use yazi_codegen::DeserializeOver2;

use super::{Offset, Origin};

#[derive(Deserialize, DeserializeOver2)]
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

impl Confirm {
	pub const fn border(&self) -> u16 { 2 }
}
