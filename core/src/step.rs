use std::{num::ParseIntError, str::FromStr};

pub enum Step {
	Fixed(isize),
	Percent(i8),
}

impl Default for Step {
	fn default() -> Self { Self::Fixed(0) }
}

impl FromStr for Step {
	type Err = ParseIntError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(if let Some(s) = s.strip_suffix('%') {
			Self::Percent(s.parse()?)
		} else {
			Self::Fixed(s.parse()?)
		})
	}
}

impl From<isize> for Step {
	fn from(n: isize) -> Self { Self::Fixed(n) }
}

impl From<usize> for Step {
	fn from(n: usize) -> Self { Self::Fixed(n as isize) }
}

impl Step {
	#[inline]
	fn fixed<F: FnOnce() -> usize>(self, f: F) -> isize {
		match self {
			Self::Fixed(n) => n,
			Self::Percent(0) => 0,
			Self::Percent(n) => n as isize * f() as isize / 100,
		}
	}

	#[inline]
	pub fn add<F: FnOnce() -> usize>(self, pos: usize, f: F) -> usize {
		let fixed = self.fixed(f);
		if fixed > 0 { pos + fixed as usize } else { pos.saturating_sub(fixed.unsigned_abs()) }
	}

	#[inline]
	pub fn is_positive(&self) -> bool {
		match *self {
			Self::Fixed(n) => n > 0,
			Self::Percent(n) => n > 0,
		}
	}
}
