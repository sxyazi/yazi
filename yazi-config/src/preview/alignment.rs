use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HorizontalAlignment {
	Left,
	#[default]
	Center,
	Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerticalAlignment {
	#[default]
	Top,
	Center,
	Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Alignment {
	#[serde(default)]
	pub horizontal: HorizontalAlignment,
	#[serde(default)]
	pub vertical:   VerticalAlignment,
}
