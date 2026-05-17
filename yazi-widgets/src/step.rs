use std::{fmt, num::ParseIntError, str::FromStr};

use serde::{Deserialize, Deserializer, de};

#[derive(Clone, Copy, Debug)]
pub enum Step {
	Top,
	Bot,
	Prev,
	Next,
	Offset(isize),
	Percent(i8),
	PercentWindow(i8),
}

impl Default for Step {
	fn default() -> Self { Self::Offset(0) }
}

impl From<isize> for Step {
	fn from(n: isize) -> Self { Self::Offset(n) }
}

impl FromStr for Step {
	type Err = ParseIntError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"top" => Self::Top,
			"bot" => Self::Bot,
			"prev" => Self::Prev,
			"next" => Self::Next,
			s if s.ends_with("%-window") => Self::PercentWindow(s[..s.len() - 8].parse()?),
			s if s.ends_with('%') => Self::Percent(s[..s.len() - 1].parse()?),
			s => Self::Offset(s.parse()?),
		})
	}
}

impl<'de> Deserialize<'de> for Step {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct Visitor;

		impl de::Visitor<'_> for Visitor {
			type Value = Step;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("a step string or integer offset")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				value.parse().map_err(E::custom)
			}

			fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				isize::try_from(value).map(Self::Value::from).map_err(E::custom)
			}

			fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				isize::try_from(value).map(Self::Value::from).map_err(E::custom)
			}
		}

		deserializer.deserialize_any(Visitor)
	}
}

impl Step {
	pub fn window_position(
		self,
		offset: usize,
		len: usize,
		limit: usize,
		scrolloff: usize,
	) -> Option<usize> {
		let Self::PercentWindow(n) = self else { return None };

		let window_len = len.saturating_sub(offset).min(limit);
		if window_len == 0 {
			return Some(0);
		}

		let scrolloff = scrolloff.min(window_len.saturating_sub(1) / 2);

		// Clamp relative position in window to not reach into any scrolloff region.
		// Still allow reaching the real list start and end if already visible.
		let pos = Self::percent_pos(offset, window_len, n);
		let min = if offset == 0 { 0 } else { offset + scrolloff };
		let max = offset + window_len - 1 - if offset + window_len >= len { 0 } else { scrolloff };

		Some(pos.clamp(min, max))
	}

	pub fn add(self, pos: usize, len: usize, limit: usize) -> usize {
		if len == 0 {
			return 0;
		}

		let off = match self {
			Self::Top => return 0,
			Self::Bot => return len - 1,
			Self::PercentWindow(n) => {
				return Self::percent_pos(0, if limit == 0 { len } else { len.min(limit) }, n);
			}
			Self::Prev => -1,
			Self::Next => 1,
			Self::Offset(n) => n,
			Self::Percent(0) => 0,
			Self::Percent(n) => n as isize * limit as isize / 100,
		};

		if matches!(self, Self::Prev | Self::Next) {
			off.saturating_add_unsigned(pos).rem_euclid(len as _) as _
		} else if off >= 0 {
			pos.saturating_add_signed(off)
		} else {
			pos.saturating_sub(off.unsigned_abs())
		}
		.min(len - 1)
	}

	fn percent_pos(offset: usize, len: usize, n: i8) -> usize {
		if len == 0 {
			return 0;
		}

		let span = len.saturating_sub(1);
		let pos = offset.saturating_add_signed(n as isize * span as isize / 100);
		pos.clamp(offset, offset + span)
	}
}
