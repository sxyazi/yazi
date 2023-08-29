use std::{collections::BTreeSet, fmt::Display};

use config::theme::{self, ColorGroup};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Mode {
	#[default]
	Normal,
	Select(usize, BTreeSet<usize>),
	Unset(usize, BTreeSet<usize>),
}

impl Mode {
	#[inline]
	pub fn color<'a>(&self, group: &'a ColorGroup) -> &'a theme::Color {
		match *self {
			Mode::Normal => &group.normal,
			Mode::Select(..) => &group.select,
			Mode::Unset(..) => &group.unset,
		}
	}

	#[inline]
	pub fn visual(&self) -> Option<(usize, &BTreeSet<usize>)> {
		match self {
			Mode::Normal => None,
			Mode::Select(start, indices) => Some((*start, indices)),
			Mode::Unset(start, indices) => Some((*start, indices)),
		}
	}

	#[inline]
	pub fn visual_mut(&mut self) -> Option<(usize, &mut BTreeSet<usize>)> {
		match self {
			Mode::Normal => None,
			Mode::Select(start, indices) => Some((*start, indices)),
			Mode::Unset(start, indices) => Some((*start, indices)),
		}
	}

	#[inline]
	pub fn pending(&self, idx: usize, state: bool) -> bool {
		match self {
			Mode::Normal => state,
			Mode::Select(_, indices) => state || indices.contains(&idx),
			Mode::Unset(_, indices) => state && !indices.contains(&idx),
		}
	}
}

impl Mode {
	#[inline]
	pub fn is_select(&self) -> bool { matches!(self, Mode::Select(..)) }

	#[inline]
	pub fn is_unset(&self) -> bool { matches!(self, Mode::Unset(..)) }

	#[inline]
	pub fn is_visual(&self) -> bool { matches!(self, Mode::Select(..) | Mode::Unset(..)) }
}

impl Display for Mode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Mode::Normal => write!(f, "NORMAL"),
			Mode::Select(..) => write!(f, "SELECT"),
			Mode::Unset(..) => write!(f, "UN-SET"),
		}
	}
}
