use crate::{completion::CompletionOpt, Position};

#[derive(Default)]
pub struct Completion {
	items:  Vec<String>,
	cursor: usize,

	pub position: Position,
	pub visible:  bool,
}

impl Completion {
	pub fn show(&mut self, opt: CompletionOpt) {
		self.close(false);
		self.visible = true;

		self.items = opt.items;
		self.position = opt.position;
	}

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

	pub fn list(&self) -> Vec<String> { self.items.clone() }
}
