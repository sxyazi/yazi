use std::{num::ParseIntError, str::FromStr};

use yazi_macro::render;
use yazi_shared::event::{CmdCow, Data};

use crate::input::{Input, op::InputOp, snap::InputSnap};

struct Opt {
	step:         OptStep,
	in_operating: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self {
			step:         c.first().and_then(|d| d.try_into().ok()).unwrap_or_default(),
			in_operating: c.bool("in-operating"),
		}
	}
}
impl From<isize> for Opt {
	fn from(step: isize) -> Self { Self { step: step.into(), in_operating: false } }
}

impl Input {
	#[yazi_codegen::command]
	pub fn r#move(&mut self, opt: Opt) {
		let snap = self.snap();
		if opt.in_operating && snap.op == InputOp::None {
			return;
		}

		render!(self.handle_op(opt.step.cursor(snap), false));

		let (limit, snap) = (self.limit, self.snap_mut());
		if snap.offset > snap.cursor {
			snap.offset = snap.cursor;
		} else if snap.value.is_empty() {
			snap.offset = 0;
		} else {
			let delta = snap.mode.delta();
			let range = snap.offset..snap.cursor + delta;
			if snap.width(range.clone()) >= limit as u16 {
				let it = snap.slice(range).chars().rev().map(|c| if snap.obscure { '•' } else { c });
				snap.offset = snap.cursor - InputSnap::find_window(it, 0, limit).end.saturating_sub(delta);
			}
		}
	}
}

// --- Step
enum OptStep {
	Offset(isize),
	Bol,
	Eol,
	FirstChar,
}

impl OptStep {
	fn cursor(self, snap: &InputSnap) -> usize {
		match self {
			Self::Offset(n) if n <= 0 => snap.cursor.saturating_add_signed(n),
			Self::Offset(n) => snap.count().min(snap.cursor + n as usize),
			Self::Bol => 0,
			Self::Eol => snap.count(),
			Self::FirstChar => {
				snap.value.chars().enumerate().find(|(_, c)| !c.is_whitespace()).map_or(0, |(i, _)| i)
			}
		}
	}
}

impl Default for OptStep {
	fn default() -> Self { 0.into() }
}

impl FromStr for OptStep {
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

impl From<isize> for OptStep {
	fn from(value: isize) -> Self { Self::Offset(value) }
}

impl TryFrom<&Data> for OptStep {
	type Error = ();

	fn try_from(value: &Data) -> Result<Self, Self::Error> {
		match value {
			Data::String(s) => s.parse().map_err(|_| ()),
			Data::Integer(i) => Ok(Self::from(*i as isize)),
			_ => Err(()),
		}
	}
}
