use std::{num::ParseIntError, str::FromStr};

use anyhow::bail;
use yazi_shared::event::{CmdCow, Data};

#[derive(Default)]
pub struct MoveOpt {
	pub step:         MoveOptStep,
	pub in_operating: bool,
}

impl From<CmdCow> for MoveOpt {
	fn from(c: CmdCow) -> Self {
		Self {
			step:         c.first().and_then(|d| d.try_into().ok()).unwrap_or_default(),
			in_operating: c.bool("in-operating"),
		}
	}
}

impl From<isize> for MoveOpt {
	fn from(step: isize) -> Self { Self { step: step.into(), in_operating: false } }
}

// --- Step
pub enum MoveOptStep {
	Offset(isize),
	Bol,
	Eol,
	FirstChar,
}

impl MoveOptStep {
	pub fn add(self, s: &str, cursor: usize) -> usize {
		match self {
			Self::Offset(n) if n <= 0 => cursor.saturating_add_signed(n),
			Self::Offset(n) => s.chars().count().min(cursor + n as usize),
			Self::Bol => 0,
			Self::Eol => s.chars().count(),
			Self::FirstChar => {
				s.chars().enumerate().find(|(_, c)| !c.is_whitespace()).map_or(0, |(i, _)| i)
			}
		}
	}
}

impl Default for MoveOptStep {
	fn default() -> Self { 0.into() }
}

impl FromStr for MoveOptStep {
	type Err = ParseIntError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"bol" => Self::Bol,
			"eol" => Self::Eol,
			"first-char" => Self::FirstChar,
			s => Self::Offset(s.parse()?),
		})
	}
}

impl From<isize> for MoveOptStep {
	fn from(value: isize) -> Self { Self::Offset(value) }
}

impl TryFrom<&Data> for MoveOptStep {
	type Error = anyhow::Error;

	fn try_from(value: &Data) -> Result<Self, Self::Error> {
		Ok(match value {
			Data::String(s) => s.parse()?,
			Data::Integer(i) => Self::from(*i as isize),
			_ => bail!("Invalid MoveOptStep data type: {value:?}"),
		})
	}
}
