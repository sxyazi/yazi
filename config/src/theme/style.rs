use ratatui::style::{self, Modifier};
use serde::{Deserialize, Serialize};

use super::Color;

#[derive(Deserialize, Serialize)]
pub struct Style {
	pub fg:        Option<Color>,
	pub bg:        Option<Color>,
	#[serde(default)]
	pub bold:      bool,
	#[serde(default)]
	pub underline: bool,
}

impl Style {
	pub fn get(&self) -> style::Style {
		let mut style = style::Style::new();

		if let Some(fg) = &self.fg {
			style = style.fg(fg.into());
		}
		if let Some(bg) = &self.bg {
			style = style.bg(bg.into());
		}
		if self.bold {
			style = style.add_modifier(Modifier::BOLD);
		}
		if self.underline {
			style = style.add_modifier(Modifier::UNDERLINED);
		}
		style
	}
}
