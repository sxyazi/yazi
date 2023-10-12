use serde::{Deserialize, Serialize};

use super::Style;

#[derive(Deserialize, Serialize)]
pub struct Help {
	pub key:  Style,
	pub exec: Style,
	pub desc: Style,
	pub curr: Style,
	pub btm:  Style,
}
