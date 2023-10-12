use serde::{Deserialize, Serialize};

use super::Style;

#[derive(Deserialize, Serialize)]
pub struct Input {
	pub border:   Style,
	pub title:    Style,
	pub value:    Style,
	pub selected: Style,
}
