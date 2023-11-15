use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize)]
pub enum Position {
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
	#[default]
	Center,
	#[serde(rename = "hovered")]
	Hovered,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "Vec<i16>")]
pub struct Offset {
	pub x:      i16,
	pub y:      i16,
	pub width:  u16,
	pub height: u16,
}

impl TryFrom<Vec<i16>> for Offset {
	type Error = anyhow::Error;

	fn try_from(values: Vec<i16>) -> Result<Self, Self::Error> {
		if values.len() != 4 {
			bail!("invalid offset: {:?}", values);
		}
		if values[2] < 0 || values[3] < 0 {
			bail!("invalid offset: {:?}", values);
		}

		Ok(Self {
			x:      values[0],
			y:      values[1],
			width:  values[2] as u16,
			height: values[3] as u16,
		})
	}
}
