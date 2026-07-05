use std::{iter::Chain, ops::Range};

use yazi_widgets::Step;

pub type VisualIndices = Chain<Range<usize>, Range<usize>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Visual {
	start: usize,
	wraps: isize,
}

impl Visual {
	pub fn new(start: usize) -> Self { Self { start, wraps: 0 } }

	pub fn arrow(&mut self, step: Step, old: usize, new: usize) {
		self.wraps += match step {
			Step::Prev if new > old => -1,
			Step::Next if new < old => 1,
			_ => 0,
		}
	}

	pub fn contains(&self, index: usize, end: usize, len: usize) -> bool {
		let (first, second) = self.ranges(end, len);
		first.contains(&index) || second.contains(&index)
	}

	pub fn indices(&self, end: usize, len: usize) -> VisualIndices {
		let (first, second) = self.ranges(end, len);
		first.chain(second)
	}

	fn ranges(&self, end: usize, len: usize) -> (Range<usize>, Range<usize>) {
		if len == 0 {
			return (0..0, 0..0);
		}

		let start = self.start.min(len - 1);
		let end = end.min(len - 1);

		match self.wraps {
			0 => (start.min(end)..start.max(end) + 1, 0..0),
			1 if start > end + 1 => (0..end + 1, start..len),
			-1 if end > start + 1 => (0..start + 1, end..len),
			_ => (0..len, 0..0),
		}
	}
}
