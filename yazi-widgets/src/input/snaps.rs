use std::mem;

use super::InputSnap;

#[derive(PartialEq, Eq)]
pub struct InputSnaps {
	idx:      usize,
	versions: Vec<InputSnap>,
	current:  InputSnap,
}

impl Default for InputSnaps {
	fn default() -> Self {
		Self {
			idx:      0,
			versions: vec![InputSnap::new(String::new(), false, 0)],
			current:  InputSnap::new(String::new(), false, 0),
		}
	}
}

impl InputSnaps {
	pub fn new(value: String, obscure: bool, limit: usize) -> Self {
		let current = InputSnap::new(value, obscure, limit);
		Self { idx: 0, versions: vec![current.clone()], current }
	}

	pub(super) fn tag(&mut self, limit: usize) -> bool {
		if self.versions.len() <= self.idx {
			return false;
		}

		// Sync *current* cursor position to the *last* version:
		// 		Save offset/cursor/ect. of the *current* as the last version,
		// 		while keeping the *last* value unchanged.
		let value = mem::take(&mut self.versions[self.idx].value);
		self.versions[self.idx] = self.current.clone();
		self.versions[self.idx].value = value;
		self.versions[self.idx].resize(limit);

		// If the *current* value is the same as the *last* version
		if self.versions[self.idx].value == self.current.value {
			return false;
		}

		self.versions.truncate(self.idx + 1);
		self.versions.push(self.current().clone());
		self.idx += 1;
		true
	}

	pub(super) fn undo(&mut self) -> bool {
		if self.idx == 0 {
			return false;
		}

		self.idx -= 1;
		self.current = self.versions[self.idx].clone();
		true
	}

	pub(super) fn redo(&mut self) -> bool {
		if self.idx + 1 >= self.versions.len() {
			return false;
		}

		self.idx += 1;
		self.current = self.versions[self.idx].clone();
		true
	}
}

impl InputSnaps {
	#[inline]
	pub fn current(&self) -> &InputSnap { &self.current }

	#[inline]
	pub(super) fn current_mut(&mut self) -> &mut InputSnap { &mut self.current }
}
