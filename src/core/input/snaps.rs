use super::{InputMode, InputSnap};

#[derive(Default, PartialEq, Eq)]
pub(super) struct InputSnaps {
	idx:      usize,
	versions: Vec<InputSnap>,
	current:  InputSnap,
}

impl InputSnaps {
	#[inline]
	pub(super) fn reset(&mut self, value: String) {
		self.idx = 0;
		self.versions.clear();
		self.versions.push(InputSnap::new(value));
		self.current = self.versions[0].clone();
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
		if self.idx + 1 == self.versions.len() {
			return false;
		}

		self.idx += 1;
		self.current = self.versions[self.idx].clone();
		true
	}

	pub(super) fn tag(&mut self) -> bool {
		if self.current.value == self.versions[self.idx].value {
			return false;
		}

		self.versions.truncate(self.idx + 1);
		self.versions.push(self.current().clone());
		self.idx += 1;
		true
	}
}

impl InputSnaps {
	#[inline]
	pub(super) fn current(&self) -> &InputSnap { &self.current }

	#[inline]
	pub(super) fn current_mut(&mut self) -> &mut InputSnap { &mut self.current }
}
