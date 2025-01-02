use unicode_width::UnicodeWidthStr;
use yazi_macro::render;
use yazi_shared::event::{CmdCow, Data};

use crate::input::{Input, op::InputOp, snap::InputSnap};

struct Opt {
	step:         Step,
	in_operating: bool,
}

enum Step {
	Offset(isize),
	Bol,
	Eol,
	FirstChar,
	LastChar,
}

impl TryFrom<&Data> for Step {
	type Error = ();

	fn try_from(d: &Data) -> Result<Self, Self::Error> {
		if let Some(offset) = d.as_isize() {
			return Ok(Step::Offset(offset));
		};
		if let Some(s) = d.as_str() {
			if let Ok(offset) = s.parse() {
				return Ok(Step::Offset(offset));
			};
			match s.as_ref() {
				"bol" => return Ok(Step::Bol),
				"eol" => return Ok(Step::Eol),
				"first-char" => return Ok(Step::FirstChar),
				"last-char" => return Ok(Step::LastChar),
				_ => (),
			}
		}
		Err(())
	}
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self {
			step: c.get(0).unwrap().try_into().unwrap(),
			in_operating: c.bool("in-operating"),
		}
	}
}
impl From<isize> for Opt {
	fn from(step: isize) -> Self {
		Self {
			step:         Step::Offset(step),
			in_operating: false,
		}
	}
}

impl Input {
	#[yazi_codegen::command]
	pub fn move_(&mut self, opt: Opt) {
		let snap = self.snap();
		if opt.in_operating && snap.op == InputOp::None {
			return;
		}

		let position = match opt.step {
			Step::Offset(offset) =>
				if offset <= 0 {
					snap.cursor.saturating_sub(offset.unsigned_abs())
				} else {
					snap.count().min(snap.cursor + offset as usize)
				},
			Step::Bol => 0,
			Step::Eol => snap.count(),
			Step::FirstChar => snap
				.value
				.chars()
				.enumerate()
				.filter(|(_, ch)| !ch.is_whitespace())
				.map(|(i, _)| i)
				.next()
				.unwrap_or(0),
			Step::LastChar => snap
				.value
				.chars()
				.rev()
				.enumerate()
				.filter(|(_, ch)| !ch.is_whitespace())
				.map(|(i, _)| i)
				.next()
				.unwrap_or(0),
		};

		render!(self.handle_op(position, false));

		let (limit, snap) = (self.limit(), self.snap_mut());
		if snap.offset > snap.cursor {
			snap.offset = snap.cursor;
		} else if snap.value.is_empty() {
			snap.offset = 0;
		} else {
			let delta = snap.mode.delta();
			let s = snap.slice(snap.offset..snap.cursor + delta);
			if s.width() >= limit {
				let s = s.chars().rev().collect::<String>();
				snap.offset = snap.cursor - InputSnap::find_window(&s, 0, limit).end.saturating_sub(delta);
			}
		}
	}
}
