use ratatui::style::{self, Modifier};
use serde::Deserialize;

use super::Color;

#[derive(Deserialize)]
pub struct Style {
	pub fg:        Option<Color>,
	pub bg:        Option<Color>,
	pub bold:      Option<bool>,
	pub underline: Option<bool>,
}

impl Style {
	pub fn get(&self) -> style::Style {
		let mut style = style::Style::new();

		if let Some(fg) = &self.fg {
			style = style.fg(fg.0);
		}
		if let Some(bg) = &self.bg {
			style = style.bg(bg.0);
		}
		if let Some(bold) = self.bold {
			if bold {
				style = style.add_modifier(Modifier::BOLD);
			} else {
				style = style.remove_modifier(Modifier::BOLD);
			}
		}
		if let Some(underline) = self.underline {
			if underline {
				style = style.add_modifier(Modifier::UNDERLINED);
			} else {
				style = style.remove_modifier(Modifier::UNDERLINED);
			}
		}
		style
	}
}
