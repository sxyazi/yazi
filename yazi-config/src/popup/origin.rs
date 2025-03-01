use std::{fmt::Display, str::FromStr};

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
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

impl FromStr for Origin {
	type Err = serde::de::value::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::deserialize(serde::de::value::StrDeserializer::new(s))
	}
}
