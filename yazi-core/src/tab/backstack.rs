#[derive(Default)]
pub struct Backstack<T> {
	cursor: usize,
	stack:  Vec<T>,
}

impl<T: Eq + Clone> Backstack<T> {
	pub fn push(&mut self, item: &T) {
		if self.stack.is_empty() {
			self.stack.push(item.clone());
			return;
		}

		if self.stack[self.cursor] == *item {
			return;
		}

		self.cursor += 1;
		if self.cursor == self.stack.len() {
			self.stack.push(item.clone());
		} else {
			self.stack[self.cursor] = item.clone();
			self.stack.truncate(self.cursor + 1);
		}

		// Only keep 30 items before the cursor, the cleanup threshold is 60
		if self.stack.len() > 60 {
			let start = self.cursor.saturating_sub(30);
			self.stack.drain(..start);
			self.cursor -= start;
		}
	}

	pub fn shift_backward(&mut self) -> Option<&T> {
		if self.cursor > 0 {
			self.cursor -= 1;
			Some(&self.stack[self.cursor])
		} else {
			None
		}
	}

	pub fn shift_forward(&mut self) -> Option<&T> {
		if self.cursor + 1 == self.stack.len() {
			None
		} else {
			self.cursor += 1;
			Some(&self.stack[self.cursor])
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_backstack() {
		let mut bs = Backstack::default();
		bs.push(&1);
		assert_eq!(bs.stack[bs.cursor], 1);

		bs.push(&2);
		bs.push(&3);
		assert_eq!(bs.stack[bs.cursor], 3);

		assert_eq!(bs.shift_backward(), Some(&2));
		assert_eq!(bs.shift_backward(), Some(&1));
		assert_eq!(bs.shift_backward(), None);
		assert_eq!(bs.shift_backward(), None);
		assert_eq!(bs.stack[bs.cursor], 1);
		assert_eq!(bs.shift_forward(), Some(&2));
		assert_eq!(bs.shift_forward(), Some(&3));
		assert_eq!(bs.shift_forward(), None);

		bs.shift_backward();
		bs.push(&4);

		assert_eq!(bs.stack[bs.cursor], 4);
		assert_eq!(bs.shift_forward(), None);
		assert_eq!(bs.shift_backward(), Some(&2));
	}
}
