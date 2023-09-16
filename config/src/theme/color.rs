use anyhow::{bail, Result};
use ratatui::style;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct Color([u8; 3]);

impl TryFrom<&str> for Color {
	type Error = anyhow::Error;

	fn try_from(s: &str) -> Result<Self, Self::Error> {
		if s.len() != 7 {
			bail!("Invalid color: {s}");
		}
		Ok(Self([
			u8::from_str_radix(&s[1..3], 16)?,
			u8::from_str_radix(&s[3..5], 16)?,
			u8::from_str_radix(&s[5..7], 16)?,
		]))
	}
}

impl TryFrom<String> for Color {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::try_from(s.as_str()) }
}

impl From<&Color> for style::Color {
	fn from(&Color(rgb): &Color) -> Self { style::Color::Rgb(rgb[0], rgb[1], rgb[2]) }
}

impl From<Color> for style::Color {
	fn from(Color(rgb): Color) -> Self { style::Color::Rgb(rgb[0], rgb[1], rgb[2]) }
}

impl Color {
	#[inline]
	pub fn fg(&self) -> style::Style { style::Style::new().fg(self.into()) }

	#[inline]
	pub fn bg(&self) -> style::Style { style::Style::new().bg(self.into()) }
}

impl Serialize for Color {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&format!("#{:02X}{:02X}{:02X}", self.0[0], self.0[1], self.0[2]))
	}
}
