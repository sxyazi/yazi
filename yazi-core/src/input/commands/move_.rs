use unicode_width::UnicodeWidthStr;
use yazi_shared::{event::{Cmd, Data}, render};

use crate::input::{op::InputOp, snap::InputSnap, Input};

pub struct Opt {
	step:         isize,
	in_operating: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self {
		Self {
			step:         c.first().and_then(Data::as_isize).unwrap_or(0),
			in_operating: c.bool("in-operating"),
		}
	}
}
impl From<isize> for Opt {
	fn from(step: isize) -> Self { Self { step, in_operating: false } }
}

impl Input {
	pub fn move_(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;

		let snap = self.snap();
		if opt.in_operating && snap.op == InputOp::None {
			return;
		}

		render!(self.handle_op(
			if opt.step <= 0 {
				snap.cursor.saturating_sub(opt.step.unsigned_abs())
			} else {
				snap.count().min(snap.cursor + opt.step as usize)
			},
			false,
		));

		let (limit, snap) = (self.limit(), self.snap_mut());
		let offset =
			snap.cursor.saturating_sub(limit / 2).min(snap.len().saturating_sub(limit.saturating_sub(1)));
		if snap.offset != offset {
			snap.offset = offset;
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
