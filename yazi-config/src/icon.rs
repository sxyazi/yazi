use serde::Deserialize;

use crate::Style;

#[derive(Clone, Debug, Deserialize)]
pub struct Icon {
	pub text:  String,
	#[serde(flatten)]
	pub style: Style,
}
