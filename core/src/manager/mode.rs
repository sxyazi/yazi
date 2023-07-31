use std::fmt::Display;

use config::theme::{self, ColorGroup};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
	#[default]
	Normal,
	Select(usize),
	Unset(usize),
}

impl Mode {
	#[inline]
	pub fn color<'a>(&self, group: &'a ColorGroup) -> &'a theme::Color {
		match *self {
			Mode::Normal => &group.normal,
			Mode::Select(_) => &group.select,
			Mode::Unset(_) => &group.unset,
		}
	}

	#[inline]
	pub fn start(&self) -> Option<usize> {
		match self {
			Mode::Normal => None,
			Mode::Select(n) => Some(*n),
			Mode::Unset(n) => Some(*n),
		}
	}
}

impl Display for Mode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Mode::Normal => write!(f, "NORMAL"),
			Mode::Select(_) => write!(f, "SELECT"),
			Mode::Unset(_) => write!(f, "UN-SET"),
		}
	}
}
