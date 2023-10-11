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
	pub progress_label:  Style,
	pub progress_normal: Style,
	pub progress_error:  Style,

	// Permissions
	pub permissions_t: Style,
	pub permissions_r: Style,
	pub permissions_w: Style,
	pub permissions_x: Style,
	pub permissions_s: Style,
}

#[derive(Deserialize, Serialize)]
pub struct StatusSeparator {
	pub opening: String,
	pub closing: String,
}
