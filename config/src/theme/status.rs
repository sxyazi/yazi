use serde::{Deserialize, Serialize};

use super::Style;

#[derive(Deserialize, Serialize)]
pub struct Status {
	pub plain:     Style,
	pub fancy:     Style,
	pub separator: StatusSeparator,

	// Mode
	pub mode_normal: Style,
	pub mode_select: Style,
	pub mode_unset:  Style,

	// Progress
	pub progress_label: Style,
	pub progress_gauge: Style,
}

#[derive(Deserialize, Serialize)]
pub struct StatusSeparator {
	pub opening: String,
	pub closing: String,
}
