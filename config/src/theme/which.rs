use serde::{Deserialize, Serialize};

use super::Style;

#[derive(Deserialize, Serialize)]
pub struct Which {
	pub mask: Style,
	pub cand: Style,
	pub rest: Style,
	pub desc: Style,

	pub separator:       String,
	pub separator_style: Style,
}
