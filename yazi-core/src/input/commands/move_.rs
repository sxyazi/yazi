use unicode_width::UnicodeWidthStr;
use yazi_macro::render;
use yazi_shared::event::{CmdCow, Data};

use crate::input::{Input, op::InputOp, snap::InputSnap};

struct Opt {
	step:         OptStep,
	in_operating: bool,
}

enum OptStep {
	Offset(isize),
	Bol,
	Eol,
	FirstChar,
	LastChar,
}

impl TryFrom<&Data> for OptStep {
	type Error = ();

	fn try_from(data: &Data) -> Result<Self, Self::Error> {
		if let Some(offset) = data.as_isize() {
			return Ok(OptStep::Offset(offset));
		};
		if let Some(string) = data.as_str() {
			if let Ok(offset) = string.parse() {
				return Ok(OptStep::Offset(offset));
			};
			match string.as_ref() {
				"bol" => return Ok(OptStep::Bol),
				"eol" => return Ok(OptStep::Eol),
				"first-char" => return Ok(OptStep::FirstChar),
				"last-char" => return Ok(OptStep::LastChar),
				_ => (),
			}
		}
		Err(())
	}
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self {
			step: c.get(0).unwrap().try_into().unwrap_or(OptStep::Offset(0)),
			in_operating: c.bool("in-operating"),
		}
	}
}
impl From<isize> for Opt {
	fn from(step: isize) -> Self {
		Self {
			step:         OptStep::Offset(step),
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
			OptStep::Offset(offset) =>
				if offset <= 0 {
					snap.cursor.saturating_sub(offset.unsigned_abs())
				} else {
					snap.count().min(snap.cursor + offset as usize)
				},
			OptStep::Bol => 0,
			OptStep::Eol => snap.count(),
			OptStep::FirstChar => snap
				.value
				.chars()
				.enumerate()
				.filter(|(_, ch)| !ch.is_whitespace())
				.map(|(i, _)| i)
				.next()
				.unwrap_or(0),
			OptStep::LastChar => snap
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
