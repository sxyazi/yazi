use std::str::FromStr;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(try_from = "String")]
pub struct Color(pub ratatui::style::Color);

impl FromStr for Color {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		ratatui::style::Color::from_str(s).map(Self).map_err(|_| anyhow::anyhow!("invalid color"))
	}
}

impl TryFrom<String> for Color {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::from_str(&s) }
}

impl From<Color> for ratatui::style::Color {
	fn from(value: Color) -> Self { value.0 }
}

impl From<ratatui::style::Color> for Color {
	fn from(value: ratatui::style::Color) -> Self { Self(value) }
}

impl Serialize for Color {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.0.to_string().serialize(serializer)
	}
}
