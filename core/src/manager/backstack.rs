pub struct BackStack<T> {
	current: usize,
	stack:   Vec<T>,
}

impl<T> BackStack<T> {
	pub fn new(item: T) -> Self { Self { current: 0, stack: vec![item] } }

	pub fn shift_backward(&mut self) -> Option<&T> {
		if self.current > 0 {
			self.current -= 1;
			Some(&self.stack[self.current])
		} else {
			None
		}
	}

	pub fn shift_forward(&mut self) -> Option<&T> {
		if self.current + 1 == self.stack.len() {
			None
		} else {
			self.current += 1;
			Some(&self.stack[self.current])
		}
	}

	pub fn push(&mut self, item: T) {
		self.current += 1;
		if self.current == self.stack.len() {
			self.stack.push(item);
		} else {
			self.stack[self.current] = item;
			self.stack.truncate(self.current + 1);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn get_current<T>(backstack: &BackStack<T>) -> &T { &backstack.stack[backstack.current] }

	#[test]
	fn test_backstack() {
		let mut backstack = BackStack::<i32>::new(1);
		assert_eq!(get_current(&backstack), &1);

		backstack.push(2);
		backstack.push(3);
		assert_eq!(get_current(&backstack), &3);

		assert_eq!(backstack.shift_backward(), Some(&2));
		assert_eq!(backstack.shift_backward(), Some(&1));
		assert_eq!(backstack.shift_backward(), None);
		assert_eq!(backstack.shift_backward(), None);
		assert_eq!(get_current(&backstack), &1);
		assert_eq!(backstack.shift_forward(), Some(&2));
		assert_eq!(backstack.shift_forward(), Some(&3));
		assert_eq!(backstack.shift_forward(), None);

		backstack.shift_backward();
		backstack.push(4);

		assert_eq!(get_current(&backstack), &4);
		assert_eq!(backstack.shift_forward(), None);
		assert_eq!(backstack.shift_backward(), Some(&2));
	}
}
