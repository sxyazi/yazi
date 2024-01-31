use yazi_shared::{event::Cmd, render};

use crate::tasks::Tasks;

pub struct Opt {
	step: isize,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self { step: c.take_first().and_then(|s| s.parse().ok()).unwrap_or(0) }
	}
}

impl Tasks {
	#[allow(clippy::should_implement_trait)]
	fn next(&mut self) {
		let limit = Self::limit().min(self.len());

		let old = self.cursor;
		self.cursor = limit.saturating_sub(1).min(self.cursor + 1);

		render!(old != self.cursor);
	}

	fn prev(&mut self) {
		let old = self.cursor;
		self.cursor = self.cursor.saturating_sub(1);

		render!(old != self.cursor);
	}

	pub fn arrow(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		if opt.step > 0 {
			self.next();
		} else {
			self.prev();
		}
	}
}
