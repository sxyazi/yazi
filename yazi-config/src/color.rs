use std::str::FromStr;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default)]
pub struct Color(ratatui::style::Color);

impl<'de> Deserialize<'de> for Color {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		ratatui::style::Color::from_str(&String::deserialize(deserializer)?)
			.map_err(serde::de::Error::custom)
			.map(Self)
	}
}

impl From<Color> for ratatui::style::Color {
	fn from(value: Color) -> Self { value.0 }
}
