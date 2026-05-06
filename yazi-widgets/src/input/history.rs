use std::sync::Mutex;

pub static INPUT_HISTORY: Mutex<InputHistory> = Mutex::new(InputHistory::new());

#[derive(Debug, Default)]
pub struct InputHistory {
	entries: Vec<String>,
	idx: Option<usize>,
	draft: String,
}

impl InputHistory {
	pub const fn new() -> Self {
		Self { entries: Vec::new(), idx: None, draft: String::new() }
	}

	pub fn push(&mut self, value: String) {
		if value.is_empty() {
			return;
		}
		if self.entries.last().map(String::as_str) != Some(&value) {
			self.entries.push(value);
		}
		self.reset();
	}

	pub fn reset(&mut self) {
		self.idx = None;
		self.draft.clear();
	}

	pub fn navigate(&mut self, step: i64, current: &str) -> Option<String> {
		if self.entries.is_empty() || step == 0 {
			return None;
		}

		let len = self.entries.len() as i64;
		let pos = self.idx.map_or(len, |i| i as i64);
		let new_pos = (pos + step).clamp(0, len);

		if new_pos == pos {
			return None;
		}

		if new_pos == len {
			self.idx = None;
			Some(std::mem::take(&mut self.draft))
		} else {
			if self.idx.is_none() {
				self.draft = current.to_owned();
			}
			self.idx = Some(new_pos as usize);
			Some(self.entries[new_pos as usize].clone())
		}
	}
}
