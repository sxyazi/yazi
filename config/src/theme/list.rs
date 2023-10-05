use serde::{Deserialize, Serialize};

use super::Style;

#[derive(Deserialize, Serialize)]
pub struct Files {
	pub hovered: Style,
}

#[derive(Deserialize, Serialize)]
pub struct Marker {
	pub selected: Style,
	pub copied:   Style,
	pub cut:      Style,
}
