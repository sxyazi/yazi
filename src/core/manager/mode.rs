use std::fmt::Display;

use crate::config::theme::{self, ColorGroup};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
	#[default]
	Normal,
	Select(usize),
	Unselect(usize),
}

impl Mode {
	#[inline]
	pub fn color<'a>(&self, group: &'a ColorGroup) -> &'a theme::Color {
		match *self {
			Mode::Normal => &group.normal,
			Mode::Select(_) => &group.select,
			Mode::Unselect(_) => &group.unselect,
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
