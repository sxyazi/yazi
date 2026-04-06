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

			fn visit_borrowed_str<E>(self, value: &'_ str) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				self.visit_str(value)
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				self.visit_str(&value)
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
	pub fn add(self, pos: usize, len: usize, limit: usize) -> usize {
		if len == 0 {
			return 0;
		}

		let off = match self {
			Self::Top => return 0,
			Self::Bot => return len - 1,
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
}
