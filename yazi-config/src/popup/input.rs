use serde::Deserialize;
use yazi_codegen::DeserializeOver2;

use super::{Offset, Origin};

#[derive(Deserialize, DeserializeOver2)]
pub struct Input {
	pub cursor_blink: bool,

	// cd
	pub cd_title:  String,
	pub cd_origin: Origin,
	pub cd_offset: Offset,

	// create
	pub create_title:  [String; 2],
	pub create_origin: Origin,
	pub create_offset: Offset,

	// rename
	pub rename_title:  String,
	pub rename_origin: Origin,
	pub rename_offset: Offset,

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
}

impl Input {
	pub const fn border(&self) -> u16 { 2 }
}
