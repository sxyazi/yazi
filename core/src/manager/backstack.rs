pub struct Backstack<T: Eq> {
	cursor: usize,
	stack:  Vec<T>,
}

impl<T: Eq> Backstack<T> {
	pub fn new(item: T) -> Self { Self { cursor: 0, stack: vec![item] } }

	pub fn push(&mut self, item: T) {
		if self.stack[self.cursor] == item {
			return;
		}

		self.cursor += 1;
		if self.cursor == self.stack.len() {
			self.stack.push(item);
		} else {
			self.stack[self.cursor] = item;
			self.stack.truncate(self.cursor + 1);
		}

		// Only keep 30 items before the cursor, the cleanup threshold is 60
		if self.stack.len() > 60 {
			let start = self.cursor.saturating_sub(30);
			self.stack.drain(..start);
			self.cursor -= start;
		}
	}

	#[cfg(test)]
	#[inline]
	pub fn current(&self) -> &T { &self.stack[self.cursor] }

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
		let mut backstack = Backstack::<u32>::new(1);
		assert_eq!(backstack.current(), &1);

		backstack.push(2);
		backstack.push(3);
		assert_eq!(backstack.current(), &3);

		assert_eq!(backstack.shift_backward(), Some(&2));
		assert_eq!(backstack.shift_backward(), Some(&1));
		assert_eq!(backstack.shift_backward(), None);
		assert_eq!(backstack.shift_backward(), None);
		assert_eq!(backstack.current(), &1);
		assert_eq!(backstack.shift_forward(), Some(&2));
		assert_eq!(backstack.shift_forward(), Some(&3));
		assert_eq!(backstack.shift_forward(), None);

		backstack.shift_backward();
		backstack.push(4);

		assert_eq!(backstack.current(), &4);
		assert_eq!(backstack.shift_forward(), None);
		assert_eq!(backstack.shift_backward(), Some(&2));
	}
}
