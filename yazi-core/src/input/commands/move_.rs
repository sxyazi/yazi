use unicode_width::UnicodeWidthStr;
use yazi_shared::Exec;

use crate::input::{op::InputOp, snap::InputSnap, Input};

pub struct Opt {
	step:         isize,
	in_operating: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self {
			step:         e.args.first().and_then(|s| s.parse().ok()).unwrap_or(0),
			in_operating: e.named.contains_key("in-operating"),
		}
	}
}
impl From<isize> for Opt {
	fn from(step: isize) -> Self { Self { step, in_operating: false } }
}

impl Input {
	pub fn move_(&mut self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into() as Opt;

		let snap = self.snap();
		if opt.in_operating && snap.op == InputOp::None {
			return false;
		}

		let b = self.handle_op(
			if opt.step <= 0 {
				snap.cursor.saturating_sub(opt.step.unsigned_abs())
			} else {
				snap.count().min(snap.cursor + opt.step as usize)
			},
			false,
		);

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

		b
	}
}
