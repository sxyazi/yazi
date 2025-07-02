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

		let o_cur = snap.cursor;
		render!(self.handle_op(opt.step.cursor(snap), false));
		let n_cur = self.snap().cursor;

		let (limit, snap) = (self.limit, self.snap_mut());
		if snap.value.is_empty() {
			return snap.offset = 0;
		}

		let (o_off, scrolloff) = (snap.offset, 5.min(limit / 2));
		snap.offset = if n_cur <= o_cur {
			let it = snap.slice(0..n_cur).chars().rev().map(|c| if snap.obscure { '•' } else { c });
			let pad = InputSnap::find_window(it, 0, scrolloff).end;

			if n_cur >= o_off { snap.offset.min(n_cur - pad) } else { n_cur - pad }
		} else {
			let count = snap.count();

			let it = snap.slice(n_cur..count).chars().map(|c| if snap.obscure { '•' } else { c });
			let pad = InputSnap::find_window(it, 0, scrolloff + snap.mode.delta()).end;

			let it = snap.slice(0..n_cur + pad).chars().rev().map(|c| if snap.obscure { '•' } else { c });
			let max = InputSnap::find_window(it, 0, limit).end;

			if snap.width(o_off..n_cur) < limit as u16 {
				snap.offset.max(n_cur + pad - max)
			} else {
				n_cur + pad - max
			}
		};
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
