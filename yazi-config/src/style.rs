use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Style {
	pub fg:          Option<Color>,
	pub bg:          Option<Color>,
	pub bold:        Option<bool>,
	pub dim:         Option<bool>,
	pub italic:      Option<bool>,
	pub underline:   Option<bool>,
	pub blink:       Option<bool>,
	pub blink_rapid: Option<bool>,
	pub reversed:    Option<bool>,
	pub hidden:      Option<bool>,
	pub crossed:     Option<bool>,
}

impl From<ratatui::style::Style> for Style {
	#[rustfmt::skip]
	fn from(value: ratatui::style::Style) -> Self {
		use ratatui::style::Modifier as M;

		let sub = value.sub_modifier;
		let all = value.add_modifier - sub;

		Self {
			fg:          value.fg,
			bg:          value.bg,
			bold:        if all.contains(M::BOLD) { Some(true) } else if sub.contains(M::BOLD) { Some(false) } else { None },
			dim:         if all.contains(M::DIM) { Some(true) } else if sub.contains(M::DIM) { Some(false) } else { None },
			italic:      if all.contains(M::ITALIC) { Some(true) } else if sub.contains(M::ITALIC) { Some(false) } else { None },
			underline:   if all.contains(M::UNDERLINED) { Some(true) } else if sub.contains(M::UNDERLINED) { Some(false) } else { None },
			blink:       if all.contains(M::SLOW_BLINK) { Some(true) } else if sub.contains(M::SLOW_BLINK) { Some(false) } else { None },
			blink_rapid: if all.contains(M::RAPID_BLINK) { Some(true) } else if sub.contains(M::RAPID_BLINK) { Some(false) } else { None },
			reversed:    if all.contains(M::REVERSED) { Some(true) } else if sub.contains(M::REVERSED) { Some(false) } else { None },
			hidden:      if all.contains(M::HIDDEN) { Some(true) } else if sub.contains(M::HIDDEN) { Some(false) } else { None },
			crossed:     if all.contains(M::CROSSED_OUT) { Some(true) } else if sub.contains(M::CROSSED_OUT) { Some(false) } else { None },
		}
	}
}

impl From<Style> for ratatui::style::Style {
	#[rustfmt::skip]
	fn from(value: Style) -> Self {
		use ratatui::style::Modifier as M;

		let (mut add, mut sub) = (M::empty(), M::empty());
		if let Some(b) = value.bold {
			if b { add |= M::BOLD } else { sub |= M::BOLD };
		}
		if let Some(b) = value.dim {
			if b { add |= M::DIM } else { sub |= M::DIM };
		}
		if let Some(b) = value.italic {
			if b { add |= M::ITALIC } else { sub |= M::ITALIC };
		}
		if let Some(b) = value.underline {
			if b { add |= M::UNDERLINED } else { sub |= M::UNDERLINED };
		}
		if let Some(b) = value.blink {
			if b { add |= M::SLOW_BLINK } else { sub |= M::SLOW_BLINK };
		}
		if let Some(b) = value.blink_rapid {
			if b { add |= M::RAPID_BLINK } else { sub |= M::RAPID_BLINK };
		}
		if let Some(b) = value.reversed {
			if b { add |= M::REVERSED } else { sub |= M::REVERSED };
		}
		if let Some(b) = value.hidden {
			if b { add |= M::HIDDEN } else { sub |= M::HIDDEN };
		}
		if let Some(b) = value.crossed {
			if b { add |= M::CROSSED_OUT } else { sub |= M::CROSSED_OUT };
		}

		Self {
			fg:              value.fg,
			bg:              value.bg,
			underline_color: None,
			add_modifier:    add,
			sub_modifier:    sub,
		}
	}
}

impl Style {
	pub fn derive(self, other: ratatui::style::Style) -> ratatui::style::Style {
		ratatui::style::Style::from(self).patch(other)
	}
}
