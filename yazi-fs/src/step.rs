use std::{num::ParseIntError, str::FromStr};

use yazi_shared::event::Data;

#[derive(Clone, Copy)]
pub enum Step {
	Top,
	Bot,
	Fixed(isize),
	Percent(i8),
}

impl Default for Step {
	fn default() -> Self { Self::Fixed(0) }
}

impl From<isize> for Step {
	fn from(n: isize) -> Self { Self::Fixed(n) }
}

impl FromStr for Step {
	type Err = ParseIntError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"top" => Self::Top,
			"bot" => Self::Bot,
			s if s.ends_with('%') => Self::Percent(s[..s.len() - 1].parse()?),
			s => Self::Fixed(s.parse()?),
		})
	}
}

impl TryFrom<&Data> for Step {
	type Error = ParseIntError;

	fn try_from(value: &Data) -> Result<Self, Self::Error> {
		Ok(match value {
			Data::Integer(i) => Self::from(*i as isize),
			Data::String(s) => s.parse()?,
			_ => "".parse()?,
		})
	}
}

impl Step {
	#[inline]
	pub fn add(self, pos: usize, limit: usize) -> usize {
		let fixed = match self {
			Self::Top => return 0,
			Self::Bot => return usize::MAX,
			Self::Fixed(n) => n,
			Self::Percent(0) => 0,
			Self::Percent(n) => n as isize * limit as isize / 100,
		};
		if fixed > 0 { pos + fixed as usize } else { pos.saturating_sub(fixed.unsigned_abs()) }
	}

	#[inline]
	pub fn is_positive(self) -> bool {
		match self {
			Self::Top | Self::Bot => false,
			Self::Fixed(n) => n > 0,
			Self::Percent(n) => n > 0,
		}
	}
}
