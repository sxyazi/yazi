use std::{ops::Deref, str::FromStr};

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default)]
pub struct Color(ratatui::style::Color);

impl Deref for Color {
	type Target = ratatui::style::Color;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Color> for ratatui::style::Color {
	fn from(value: Color) -> Self { value.0 }
}

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
