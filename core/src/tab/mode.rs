use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Mode {
	#[default]
	Normal,
	Select(usize, BTreeSet<usize>),
	Unset(usize, BTreeSet<usize>),
}

impl Mode {
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
