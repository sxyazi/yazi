use yazi_shared::{event::Cmd, render};

use crate::help::Help;

pub struct Opt {
	step: isize,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self { step: c.take_first_str().and_then(|s| s.parse().ok()).unwrap_or(0) }
	}
}
impl From<isize> for Opt {
	fn from(step: isize) -> Self { Self { step } }
}

impl Help {
	#[inline]
	pub fn arrow(&mut self, opt: impl Into<Opt>) {
		let max = self.bindings.len().saturating_sub(1);
		self.offset = self.offset.min(max);
		self.cursor = self.cursor.min(max);

		let opt = opt.into() as Opt;
		if opt.step > 0 {
			self.next(opt.step as usize);
		} else {
			self.prev(opt.step.unsigned_abs());
		}
	}

	fn next(&mut self, step: usize) {
		let len = self.bindings.len();
		if len == 0 {
			return;
		}

		let old = self.cursor;
		self.cursor = (self.cursor + step).min(len - 1);

		let limit = Self::limit();
		if self.cursor >= (self.offset + limit).min(len).saturating_sub(5) {
			self.offset = len.saturating_sub(limit).min(self.offset + self.cursor - old);
		}

		render!(old != self.cursor);
	}

	fn prev(&mut self, step: usize) {
		let old = self.cursor;
		self.cursor = self.cursor.saturating_sub(step);

		if self.cursor < self.offset + 5 {
			self.offset = self.offset.saturating_sub(old - self.cursor);
		}

		render!(old != self.cursor);
	}
}
