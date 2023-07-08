use anyhow::Result;
use ratatui::style;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Color {
	pub fg: String,
	pub bg: String,
}

impl Color {
	pub fn fg_rgb(&self) -> style::Color {
		if self.fg.len() < 7 {
			return style::Color::Reset;
		}
		let convert = || -> Result<style::Color> {
			Ok(style::Color::Rgb(
				u8::from_str_radix(&self.fg[1..3], 16)?,
				u8::from_str_radix(&self.fg[3..5], 16)?,
				u8::from_str_radix(&self.fg[5..7], 16)?,
			))
		};
		convert().unwrap_or(style::Color::Reset)
	}

	pub fn bg_rgb(&self) -> style::Color {
		if self.bg.len() < 7 {
			return style::Color::Reset;
		}
		let convert = || -> Result<style::Color> {
			Ok(style::Color::Rgb(
				u8::from_str_radix(&self.bg[1..3], 16)?,
				u8::from_str_radix(&self.bg[3..5], 16)?,
				u8::from_str_radix(&self.bg[5..7], 16)?,
			))
		};
		convert().unwrap_or(style::Color::Reset)
	}
}
