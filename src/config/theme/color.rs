use std::ops::Deref;

use anyhow::{bail, Result};
use ratatui::style;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct Color(pub(super) style::Color);

impl Default for Color {
	fn default() -> Self { Self(style::Color::Reset) }
}

impl TryFrom<String> for Color {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> {
		if s.len() < 7 {
			bail!("Invalid color: {}", s);
		}
		Ok(Self(style::Color::Rgb(
			u8::from_str_radix(&s[1..3], 16)?,
			u8::from_str_radix(&s[3..5], 16)?,
			u8::from_str_radix(&s[5..7], 16)?,
		)))
	}
}

impl Deref for Color {
	type Target = style::Color;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Color {
	pub fn fg(&self) -> style::Style { style::Style::new().fg(self.0) }

	pub fn bg(&self) -> style::Style { style::Style::new().bg(self.0) }
}

#[derive(Deserialize)]
pub struct ColorGroup {
	pub normal: Color,
	pub select: Color,
	pub unset:  Color,
}
