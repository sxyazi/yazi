use std::fmt::Display;

use crate::config::{theme, THEME};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
	#[default]
	Normal,
	Select(usize),
	Unselect(usize),
}

impl Mode {
	#[inline]
	pub fn color(&self) -> &theme::Color {
		match *self {
			Mode::Normal => &THEME.mode.normal,
			Mode::Select(_) => &THEME.mode.select,
			Mode::Unselect(_) => &THEME.mode.unselect,
		}
	}

	#[inline]
	pub fn start(&self) -> Option<usize> {
		match self {
			Mode::Normal => None,
			Mode::Select(n) => Some(*n),
			Mode::Unselect(n) => Some(*n),
		}
	}
}

impl Display for Mode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Mode::Normal => write!(f, "NORMAL"),
			Mode::Select(_) => write!(f, "SELECT"),
			Mode::Unselect(_) => write!(f, "UN-SEL"),
		}
	}
}
