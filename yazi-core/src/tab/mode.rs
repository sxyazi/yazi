use std::{collections::BTreeSet, fmt::Display, mem};

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
			Self::Normal => None,
			Self::Select(start, indices) => Some((*start, indices)),
			Self::Unset(start, indices) => Some((*start, indices)),
		}
	}

	pub fn take_visual(&mut self) -> Option<(usize, BTreeSet<usize>)> {
		match mem::take(self) {
			Self::Normal => None,
			Self::Select(start, indices) => Some((start, indices)),
			Self::Unset(start, indices) => Some((start, indices)),
		}
	}
}

impl Mode {
	pub fn is_select(&self) -> bool { matches!(self, Self::Select(..)) }

	pub fn is_unset(&self) -> bool { matches!(self, Self::Unset(..)) }

	pub fn is_visual(&self) -> bool { matches!(self, Self::Select(..) | Self::Unset(..)) }
}

impl Display for Mode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Self::Normal => "normal",
			Self::Select(..) => "select",
			Self::Unset(..) => "unset",
		})
	}
}
