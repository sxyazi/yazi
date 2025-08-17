use yazi_shared::url::{Url, UrlBuf};

#[derive(Default)]
pub struct Backstack {
	cursor: usize,
	stack:  Vec<UrlBuf>,
}

impl Backstack {
	pub fn push(&mut self, url: Url) {
		if self.stack.is_empty() {
			self.stack.push(url.to_owned());
			return;
		}

		if self.stack[self.cursor] == url {
			return;
		}

		self.cursor += 1;
		if self.cursor == self.stack.len() {
			self.stack.push(url.to_owned());
		} else {
			self.stack[self.cursor] = url.to_owned();
			self.stack.truncate(self.cursor + 1);
		}

		// Only keep 30 URLs before the cursor, the cleanup threshold is 60
		if self.stack.len() > 60 {
			let start = self.cursor.saturating_sub(30);
			self.stack.drain(..start);
			self.cursor -= start;
		}
	}

	pub fn shift_backward(&mut self) -> Option<&UrlBuf> {
		if self.cursor > 0 {
			self.cursor -= 1;
			Some(&self.stack[self.cursor])
		} else {
			None
		}
	}

	pub fn shift_forward(&mut self) -> Option<&UrlBuf> {
		if self.cursor + 1 >= self.stack.len() {
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
		let mut bs: Backstack = Backstack::default();
		assert_eq!(bs.shift_forward(), None);

		bs.push(Url::regular("1"));
		assert_eq!(bs.stack[bs.cursor], Url::regular("1"));

		bs.push(Url::regular("2"));
		bs.push(Url::regular("3"));
		assert_eq!(bs.stack[bs.cursor], Url::regular("3"));

		assert_eq!(bs.shift_backward().unwrap(), Url::regular("2"));
		assert_eq!(bs.shift_backward().unwrap(), Url::regular("1"));
		assert_eq!(bs.shift_backward(), None);
		assert_eq!(bs.shift_backward(), None);
		assert_eq!(bs.stack[bs.cursor], Url::regular("1"));
		assert_eq!(bs.shift_forward().unwrap(), Url::regular("2"));
		assert_eq!(bs.shift_forward().unwrap(), Url::regular("3"));
		assert_eq!(bs.shift_forward(), None);

		bs.shift_backward();
		bs.push(Url::regular("4"));

		assert_eq!(bs.stack[bs.cursor], Url::regular("4"));
		assert_eq!(bs.shift_forward(), None);
		assert_eq!(bs.shift_backward().unwrap(), Url::regular("2"));
	}
}
