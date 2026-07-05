use std::{fmt::{self, Display}, mem};

use strum::EnumIs;

use super::Visual;
use crate::tab::VisualIndices;

#[derive(Clone, Debug, Default, EnumIs, Eq, PartialEq)]
pub enum Mode {
	#[default]
	Normal,
	Select(Visual),
	Unset(Visual),
}

impl Mode {
	pub fn visual(&self) -> Option<Visual> {
		match self {
			Self::Normal => None,
			Self::Select(visual) | Self::Unset(visual) => Some(*visual),
		}
	}

	pub fn visual_mut(&mut self) -> Option<&mut Visual> {
		match self {
			Self::Normal => None,
			Self::Select(visual) | Self::Unset(visual) => Some(visual),
		}
	}

	pub fn take_visual(&mut self, end: usize, len: usize) -> Option<VisualIndices> {
		match mem::take(self) {
			Self::Normal => None,
			Self::Select(visual) | Self::Unset(visual) => Some(visual.indices(end, len)),
		}
	}
}

impl Display for Mode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			Self::Normal => "normal",
			Self::Select(..) => "select",
			Self::Unset(..) => "unset",
		})
	}
}
