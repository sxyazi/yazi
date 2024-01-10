use ratatui::style::Modifier;
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};

use super::Color;

#[derive(Clone, Copy, Deserialize)]
#[serde(from = "StyleShadow")]
pub struct Style {
	pub fg:       Option<Color>,
	pub bg:       Option<Color>,
	pub modifier: Modifier,
}

impl Serialize for Style {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut map = serializer.serialize_map(Some(3))?;
		map.serialize_entry("fg", &self.fg)?;
		map.serialize_entry("bg", &self.bg)?;
		map.serialize_entry("modifier", &self.modifier.bits())?;
		map.end()
	}
}

impl From<Style> for ratatui::style::Style {
	fn from(value: Style) -> Self {
		ratatui::style::Style {
			fg:              value.fg.map(Into::into),
			bg:              value.bg.map(Into::into),
			underline_color: None,
			add_modifier:    value.modifier,
			sub_modifier:    Modifier::empty(),
		}
	}
}

#[derive(Default, Deserialize)]
pub(super) struct StyleShadow {
	#[serde(default)]
	pub(super) fg:          Option<Color>,
	#[serde(default)]
	pub(super) bg:          Option<Color>,
	#[serde(default)]
	pub(super) bold:        bool,
	#[serde(default)]
	pub(super) dim:         bool,
	#[serde(default)]
	pub(super) italic:      bool,
	#[serde(default)]
	pub(super) underline:   bool,
	#[serde(default)]
	pub(super) blink:       bool,
	#[serde(default)]
	pub(super) blink_rapid: bool,
	#[serde(default)]
	pub(super) reversed:    bool,
	#[serde(default)]
	pub(super) hidden:      bool,
	#[serde(default)]
	pub(super) crossed:     bool,
}

impl From<StyleShadow> for Style {
	fn from(value: StyleShadow) -> Self {
		let mut modifier = Modifier::empty();
		if value.bold {
			modifier |= Modifier::BOLD;
		}
		if value.dim {
			modifier |= Modifier::DIM;
		}
		if value.italic {
			modifier |= Modifier::ITALIC;
		}
		if value.underline {
			modifier |= Modifier::UNDERLINED;
		}
		if value.blink {
			modifier |= Modifier::SLOW_BLINK;
		}
		if value.blink_rapid {
			modifier |= Modifier::RAPID_BLINK;
		}
		if value.reversed {
			modifier |= Modifier::REVERSED;
		}
		if value.hidden {
			modifier |= Modifier::HIDDEN;
		}
		if value.crossed {
			modifier |= Modifier::CROSSED_OUT;
		}

		Self { fg: value.fg, bg: value.bg, modifier }
	}
}
