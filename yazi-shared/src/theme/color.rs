use std::str::FromStr;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct Color(ratatui::style::Color);

impl FromStr for Color {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		ratatui::style::Color::from_str(s).map(Self).map_err(|_| anyhow!("invalid color: {s}"))
	}
}

impl TryFrom<String> for Color {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::from_str(&s) }
}

impl From<Color> for ratatui::style::Color {
	fn from(value: Color) -> Self { value.0 }
}

impl Serialize for Color {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.0.to_string().serialize(serializer)
	}
}
