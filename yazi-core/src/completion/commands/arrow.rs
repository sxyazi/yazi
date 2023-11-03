use yazi_config::keymap::Exec;

use crate::completion::Completion;

pub struct Opt(isize);

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self(e.args.first().and_then(|s| s.parse().ok()).unwrap_or(0)) }
}

impl Completion {
	fn next(&mut self, step: usize) -> bool {
		let len = self.cands.len();
		if len == 0 {
			return false;
		}

		let old = self.cursor;
		self.cursor = (self.cursor + step).min(len - 1);

		let limit = self.limit();
		if self.cursor >= len.min(self.offset + limit) {
			self.offset = len.saturating_sub(limit).min(self.offset + self.cursor - old);
		}

		old != self.cursor
	}

	fn prev(&mut self, step: usize) -> bool {
		let old = self.cursor;
		self.cursor = self.cursor.saturating_sub(step);

		if self.cursor < self.offset {
			self.offset = self.offset.saturating_sub(old - self.cursor);
		}

		old != self.cursor
	}

	pub fn arrow(&mut self, opt: impl Into<Opt>) -> bool {
		let step = opt.into().0;
		if step > 0 { self.next(step as usize) } else { self.prev(step.unsigned_abs()) }
	}
}
