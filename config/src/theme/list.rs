use serde::{Deserialize, Serialize};

use super::Style;

#[derive(Deserialize, Serialize)]
pub struct Selection {
	pub hovered: Style,
}

#[derive(Deserialize, Serialize)]
pub struct Marker {
	pub selecting: Style,
	pub selected:  Style,
}
