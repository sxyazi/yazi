use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::Style;

#[derive(Deserialize, Serialize, Validate)]
pub struct Tabs {
	pub active:    Style,
	pub inactive:  Style,
	#[validate(range(min = 1, message = "Must be greater than 0"))]
	pub max_width: u8,
}

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

#[derive(Deserialize, Serialize)]
pub struct Preview {
	pub hovered:       Style,
	pub syntect_theme: PathBuf,
}
