use crate::{completion::CompletionOpt, Position};

#[derive(Default)]
pub struct Completion {
	pub items:  Vec<String>,
	pub cursor: usize,

	pub identifier: String,
	pub visible:    bool,

	// TODO: remove these
	pub position:   Position,
	pub column_cnt: u8,
	pub max_width:  u16,
}

impl Completion {
	pub fn show(&mut self, opt: CompletionOpt) {
		self.close();
		self.items = opt.items;

		self.identifier = format!(
			"{}",
			std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis()
		);
		self.visible = true;

		// TODO: remove these
		self.position = opt.position;
		self.column_cnt = opt.column_cnt;
		self.max_width = opt.max_width;
	}

	pub fn close(&mut self) -> bool {
		self.cursor = 0;

		self.identifier = String::new();
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
