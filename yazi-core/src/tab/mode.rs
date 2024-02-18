use std::{collections::BTreeSet, mem};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Mode {
	#[default]
	Normal,
	Select(usize, BTreeSet<usize>),
	Unset(usize, BTreeSet<usize>),
}

impl Mode {
	pub fn visual_mut(&mut self) -> Option<(usize, &mut BTreeSet<usize>)> {
		match self {
			Mode::Normal => None,
			Mode::Select(start, indices) => Some((*start, indices)),
			Mode::Unset(start, indices) => Some((*start, indices)),
		}
	}

	pub fn take_visual(&mut self) -> Option<(usize, BTreeSet<usize>)> {
		match mem::take(self) {
			Mode::Normal => None,
			Mode::Select(start, indices) => Some((start, indices)),
			Mode::Unset(start, indices) => Some((start, indices)),
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

impl ToString for Mode {
	fn to_string(&self) -> String {
		match self {
			Mode::Normal => "normal",
			Mode::Select(..) => "select",
			Mode::Unset(..) => "unset",
		}
		.to_string()
	}
}
