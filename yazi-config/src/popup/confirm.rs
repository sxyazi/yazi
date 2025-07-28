use serde::Deserialize;
use yazi_codegen::DeserializeOver2;

use super::{Offset, Origin};
use crate::popup::Position;

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
	pub overwrite_title:  String,
	pub overwrite_body:   String,
	pub overwrite_origin: Origin,
	pub overwrite_offset: Offset,

	// quit
	pub quit_title:  String,
	pub quit_body:   String,
	pub quit_origin: Origin,
	pub quit_offset: Offset,
}

impl Confirm {
	pub const fn border(&self) -> u16 { 2 }

	pub const fn trash_position(&self) -> Position {
		Position::new(self.trash_origin, self.trash_offset)
	}

	pub const fn delete_position(&self) -> Position {
		Position::new(self.delete_origin, self.delete_offset)
	}

	pub const fn overwrite_position(&self) -> Position {
		Position::new(self.overwrite_origin, self.overwrite_offset)
	}

	pub const fn quit_position(&self) -> Position {
		Position::new(self.quit_origin, self.quit_offset)
	}
}
