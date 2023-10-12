use serde::{Deserialize, Serialize};

use super::Style;

#[derive(Deserialize, Serialize)]
pub struct Help {
	pub on:   Style,
	pub exec: Style,
	pub desc: Style,

	pub hovered: Style,
	pub footer:  Style,
}
