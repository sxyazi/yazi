use serde::Deserialize;
use yazi_binding::position::{Offset, Origin, Position};
use yazi_codegen::{DeserializeOver, DeserializeOver2};

#[derive(Deserialize, DeserializeOver, DeserializeOver2)]
pub struct Confirm {
	// trash
	pub(crate) trash_title: String,
	trash_origin:           Origin,
	trash_offset:           Offset,

	// delete
	pub(crate) delete_title: String,
	delete_origin:           Origin,
	delete_offset:           Offset,

	// overwrite
	pub(crate) overwrite_title: String,
	pub(crate) overwrite_body:  String,
	overwrite_origin:           Origin,
	overwrite_offset:           Offset,

	// quit
	pub(crate) quit_title: String,
	pub(crate) quit_body:  String,
	quit_origin:           Origin,
	quit_offset:           Offset,
}

impl Confirm {
	pub(crate) const fn trash_position(&self) -> Position {
		Position::new(self.trash_origin, self.trash_offset)
	}

	pub(crate) const fn delete_position(&self) -> Position {
		Position::new(self.delete_origin, self.delete_offset)
	}

	pub(crate) const fn overwrite_position(&self) -> Position {
		Position::new(self.overwrite_origin, self.overwrite_offset)
	}

	pub(crate) const fn quit_position(&self) -> Position {
		Position::new(self.quit_origin, self.quit_offset)
	}
}
