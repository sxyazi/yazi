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

	pub fn navigate(&mut self, up: bool, current: &str) -> Option<String> {
		if self.entries.is_empty() {
			return None;
		}

		if up {
			let new_idx = match self.idx {
				None => {
					self.draft = current.to_owned();
					self.entries.len() - 1
				}
				Some(0) => return None,
				Some(i) => i - 1,
			};
			self.idx = Some(new_idx);
			Some(self.entries[new_idx].clone())
		} else {
			match self.idx {
				None => None,
				Some(i) if i + 1 >= self.entries.len() => {
					self.idx = None;
					let draft = std::mem::take(&mut self.draft);
					Some(draft)
				}
				Some(i) => {
					self.idx = Some(i + 1);
					Some(self.entries[i + 1].clone())
				}
			}
		}
	}
}
