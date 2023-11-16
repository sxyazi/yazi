use serde::Deserialize;

#[derive(Clone, Copy, Default, Deserialize, PartialEq, Eq)]
pub enum Origin {
	#[default]
	#[serde(rename = "top-left")]
	TopLeft,
	#[serde(rename = "top-center")]
	TopCenter,
	#[serde(rename = "top-right")]
	TopRight,

	#[serde(rename = "bottom-left")]
	BottomLeft,
	#[serde(rename = "bottom-center")]
	BottomCenter,
	#[serde(rename = "bottom-right")]
	BottomRight,

	#[serde(rename = "center")]
	Center,
	#[serde(rename = "hovered")]
	Hovered,
}
