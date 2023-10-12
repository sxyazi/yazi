use serde::{Deserialize, Serialize};

use super::Style;

#[derive(Deserialize, Serialize)]
pub struct Select {
	pub border:   Style,
	pub active:   Style,
	pub inactive: Style,
}
