use std::ops::Deref;

use mlua::{Lua, Table};
use serde::Deserialize;
use strum::FromRepr;

#[derive(Clone, Copy, Debug, Default, Deserialize, FromRepr)]
#[repr(u8)]
#[serde(rename_all = "kebab-case")]
pub enum BorderType {
	#[default]
	Plain,
	Rounded,
	Double,
	Thick,
	QuadrantInside,
	QuadrantOutside,
}

impl Deref for BorderType {
	type Target = ratatui_widgets::borders::BorderType;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Plain => &Self::Target::Plain,
			Self::Rounded => &Self::Target::Rounded,
			Self::Double => &Self::Target::Double,
			Self::Thick => &Self::Target::Thick,
			Self::QuadrantInside => &Self::Target::QuadrantInside,
			Self::QuadrantOutside => &Self::Target::QuadrantOutside,
		}
	}
}

impl From<u8> for BorderType {
	fn from(value: u8) -> Self { Self::from_repr(value).unwrap_or_default() }
}

impl BorderType {
	// TODO: remove
	pub fn compose(lua: &Lua) -> mlua::Result<Table> {
		lua.create_table_from([
			("PLAIN", 0),
			("ROUNDED", 1),
			("DOUBLE", 2),
			("THICK", 3),
			("QUADRANT_INSIDE", 4),
			("QUADRANT_OUTSIDE", 5),
		])
	}
}
