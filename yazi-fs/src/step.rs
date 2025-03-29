use std::{num::ParseIntError, str::FromStr};

use yazi_shared::event::Data;

#[derive(Clone, Copy, Debug)]
pub enum Step {
	Top,
	Bot,
	Prev,
	Next,
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
			"prev" => Self::Prev,
			"next" => Self::Next,
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
	pub fn add(self, pos: usize, len: usize, limit: usize) -> usize {
		if len == 0 {
			return 0;
		}

		let fixed = match self {
			Self::Top => return 0,
			Self::Bot => return len - 1,
			Self::Prev => -1,
			Self::Next => 1,
			Self::Fixed(n) => n,
			Self::Percent(0) => 0,
			Self::Percent(n) => n as isize * limit as isize / 100,
		};

		if matches!(self, Self::Prev | Self::Next) {
			fixed.saturating_add_unsigned(pos).rem_euclid(len as _) as _
		} else if fixed >= 0 {
			pos.saturating_add_signed(fixed)
		} else {
			pos.saturating_sub(fixed.unsigned_abs())
		}
		.min(len - 1)
	}
}
