use yazi_shared::event::Exec;

use crate::tasks::Tasks;

pub struct Opt {
	step: isize,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self { step: e.args.first().and_then(|s| s.parse().ok()).unwrap_or(0) }
	}
}

impl Tasks {
	#[allow(clippy::should_implement_trait)]
	fn next(&mut self) -> bool {
		let limit = Self::limit().min(self.len());

		let old = self.cursor;
		self.cursor = limit.saturating_sub(1).min(self.cursor + 1);

		old != self.cursor
	}

	fn prev(&mut self) -> bool {
		let old = self.cursor;
		self.cursor = self.cursor.saturating_sub(1);
		old != self.cursor
	}

	pub fn arrow(&mut self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into() as Opt;
		if opt.step > 0 { self.next() } else { self.prev() }
	}
}
