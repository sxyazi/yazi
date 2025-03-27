use yazi_fs::Step;
use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::pick::Pick;

struct Opt {
	step: Step,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self { step: c.first().and_then(|d| d.try_into().ok()).unwrap_or_default() }
	}
}

impl Pick {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: Opt) {
		let len = self.items.len();
		if len == 0 {
			return;
		}

		let new = opt.step.add(self.cursor, len, self.limit());
		if new > self.cursor {
			self.next(new);
		} else {
			self.prev(new);
		}
	}

	fn next(&mut self, new: usize) {
		let len = self.items.len();
		let old = self.cursor;
		self.cursor = new.min(len - 1);

		let limit = self.limit();
		if self.cursor >= len.min(self.offset + limit) {
			self.offset = len.saturating_sub(limit).min(self.offset + self.cursor - old);
		}

		render!(old != self.cursor);
	}

	fn prev(&mut self, new: usize) {
		let old = self.cursor;
		self.cursor = new.min(self.items.len().saturating_sub(1));

		if self.cursor < self.offset {
			self.offset = self.offset.saturating_sub(old - self.cursor);
		}

		render!(old != self.cursor);
	}
}
