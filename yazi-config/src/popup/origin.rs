use std::{fmt::Display, str::FromStr};

use anyhow::bail;
use serde::Deserialize;

#[derive(Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String")]
pub enum Origin {
	#[default]
	TopLeft,
	TopCenter,
	TopRight,

	BottomLeft,
	BottomCenter,
	BottomRight,

	Center,
	Hovered,
}

impl FromStr for Origin {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"top-left" => Self::TopLeft,
			"top-center" => Self::TopCenter,
			"top-right" => Self::TopRight,

			"bottom-left" => Self::BottomLeft,
			"bottom-center" => Self::BottomCenter,
			"bottom-right" => Self::BottomRight,

			"center" => Self::Center,
			"hovered" => Self::Hovered,
			_ => bail!("Invalid `origin` value: {s}"),
		})
	}
}

impl TryFrom<String> for Origin {
	type Error = anyhow::Error;

	fn try_from(value: String) -> Result<Self, Self::Error> { Self::from_str(&value) }
}

impl Display for Origin {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Self::TopLeft => "top-left",
			Self::TopCenter => "top-center",
			Self::TopRight => "top-right",

			Self::BottomLeft => "bottom-left",
			Self::BottomCenter => "bottom-center",
			Self::BottomRight => "bottom-right",

			Self::Center => "center",
			Self::Hovered => "hovered",
		})
	}
}
