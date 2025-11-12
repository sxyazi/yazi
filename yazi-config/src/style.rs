use ratatui::style::Modifier;
use serde::Deserialize;

use crate::Color;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct Style {
	pub fg:          Option<Color>,
	pub bg:          Option<Color>,
	#[serde(default)]
	pub bold:        bool,
	#[serde(default)]
	pub dim:         bool,
	#[serde(default)]
	pub italic:      bool,
	#[serde(default)]
	pub underline:   bool,
	#[serde(default)]
	pub blink:       bool,
	#[serde(default)]
	pub blink_rapid: bool,
	#[serde(default)]
	pub reversed:    bool,
	#[serde(default)]
	pub hidden:      bool,
	#[serde(default)]
	pub crossed:     bool,
}

impl From<Style> for ratatui::style::Style {
	fn from(value: Style) -> Self {
		Self {
			fg:              value.fg.map(Into::into),
			bg:              value.bg.map(Into::into),
			underline_color: None,
			add_modifier:    value.into(),
			sub_modifier:    Modifier::empty(),
		}
	}
}

impl From<Style> for ratatui::style::Modifier {
	fn from(value: Style) -> Self {
		let mut modifier = Self::empty();
		if value.bold {
			modifier |= Self::BOLD;
		}
		if value.dim {
			modifier |= Self::DIM;
		}
		if value.italic {
			modifier |= Self::ITALIC;
		}
		if value.underline {
			modifier |= Self::UNDERLINED;
		}
		if value.blink {
			modifier |= Self::SLOW_BLINK;
		}
		if value.blink_rapid {
			modifier |= Self::RAPID_BLINK;
		}
		if value.reversed {
			modifier |= Self::REVERSED;
		}
		if value.hidden {
			modifier |= Self::HIDDEN;
		}
		if value.crossed {
			modifier |= Self::CROSSED_OUT;
		}
		modifier
	}
}

impl Style {
	pub fn derive(self, other: ratatui::style::Style) -> ratatui::style::Style {
		ratatui::style::Style::from(self).patch(other)
	}
}
