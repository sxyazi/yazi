use serde::{Deserialize, Serialize};

use super::{Color, Style};

#[derive(Deserialize, Serialize)]
pub struct Which {
	pub block:     Style,
	pub key:       Style,
	pub separator: WhichSeparator,
	pub extend:    Style,
	pub desc:      Style,
}

#[derive(Deserialize, Serialize)]
pub struct WhichItem {
	pub key:         Color,
	pub extend:      Color,
	pub description: Color,
}

#[derive(Deserialize, Serialize)]
pub struct WhichSeparator {
	pub separator: String,
	pub color:     Color,
}
