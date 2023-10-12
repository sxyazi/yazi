use serde::{Deserialize, Serialize};

use super::Style;

#[derive(Deserialize, Serialize)]
pub struct Tasks {
	pub border:  Style,
	pub title:   Style,
	pub hovered: Style,
}
