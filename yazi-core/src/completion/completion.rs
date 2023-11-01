#[derive(Default)]
pub struct Completion {
	pub items:  Vec<String>,
	pub cursor: usize,

	pub version: usize,
	pub visible: bool,
}

impl Completion {
	pub fn close(&mut self, submit: bool) -> bool {
		self.cursor = 0;

		self.visible = false;
		true
	}

	pub fn next(&mut self, step: usize) -> bool {
		let len = self.items.len();
		if len == 0 {
			return false;
		}

		let old = self.cursor;
		self.cursor = (self.cursor + step).min(len - 1);

		old != self.cursor
	}

	pub fn prev(&mut self, step: usize) -> bool {
		let old = self.cursor;
		self.cursor = self.cursor.saturating_sub(step);

		old != self.cursor
	}

	#[inline]
	pub fn selected(&self) -> Option<&String> { self.items.get(self.cursor) }
}
