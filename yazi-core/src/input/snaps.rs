use std::mem;

use super::InputSnap;

#[derive(Default, PartialEq, Eq)]
pub(super) struct InputSnaps {
	idx:      usize,
	versions: Vec<InputSnap>,
	current:  InputSnap,
}

impl InputSnaps {
	#[inline]
	pub(super) fn reset(&mut self, value: String, limit: usize) {
		self.idx = 0;
		self.versions.clear();
		self.versions.push(InputSnap::new(value, limit));
		self.current = self.versions[0].clone();
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
		self.versions[self.idx].reset(limit);

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
	pub(super) fn current(&self) -> &InputSnap { &self.current }

	#[inline]
	pub(super) fn current_mut(&mut self) -> &mut InputSnap { &mut self.current }
}
