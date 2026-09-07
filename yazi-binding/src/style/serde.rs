use ratatui_core::style::{Color, Modifier};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Style;

impl<'de> Deserialize<'de> for Style {
	fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
		Ok(Self(Flat::deserialize(de)?.into()))
	}
}

impl Serialize for Style {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		Flat::from(self.0).serialize(serializer)
	}
}

// --- Flat
#[derive(Deserialize, Serialize)]
struct Flat {
	fg:          Option<Color>,
	bg:          Option<Color>,
	bold:        Option<bool>,
	dim:         Option<bool>,
	italic:      Option<bool>,
	underline:   Option<bool>,
	blink:       Option<bool>,
	blink_rapid: Option<bool>,
	reversed:    Option<bool>,
	hidden:      Option<bool>,
	crossed:     Option<bool>,
}

impl From<ratatui_core::style::Style> for Flat {
	fn from(value: ratatui_core::style::Style) -> Self {
		let sub = value.sub_modifier;
		let add = value.add_modifier - sub;
		let opt = |m| add.contains(m).then_some(true).or(sub.contains(m).then_some(false));

		Self {
			fg:          value.fg,
			bg:          value.bg,
			bold:        opt(Modifier::BOLD),
			dim:         opt(Modifier::DIM),
			italic:      opt(Modifier::ITALIC),
			underline:   opt(Modifier::UNDERLINED),
			blink:       opt(Modifier::SLOW_BLINK),
			blink_rapid: opt(Modifier::RAPID_BLINK),
			reversed:    opt(Modifier::REVERSED),
			hidden:      opt(Modifier::HIDDEN),
			crossed:     opt(Modifier::CROSSED_OUT),
		}
	}
}

impl From<Flat> for ratatui_core::style::Style {
	fn from(value: Flat) -> Self {
		let mut style = Self { fg: value.fg, bg: value.bg, ..Default::default() };

		for (state, modifier) in [
			(value.bold, Modifier::BOLD),
			(value.dim, Modifier::DIM),
			(value.italic, Modifier::ITALIC),
			(value.underline, Modifier::UNDERLINED),
			(value.blink, Modifier::SLOW_BLINK),
			(value.blink_rapid, Modifier::RAPID_BLINK),
			(value.reversed, Modifier::REVERSED),
			(value.hidden, Modifier::HIDDEN),
			(value.crossed, Modifier::CROSSED_OUT),
		] {
			if let Some(b) = state {
				style = if b { style.add_modifier(modifier) } else { style.remove_modifier(modifier) };
			}
		}

		style
	}
}
