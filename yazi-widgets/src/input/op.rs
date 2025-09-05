use std::ops::Range;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputOp {
	#[default]
	None,
	Select(usize),
	Delete(bool, bool, usize), // cut, insert, start
	Yank(usize),
}

impl InputOp {
	#[inline]
	pub(super) fn start(&self) -> Option<usize> {
		match self {
			Self::None => None,
			Self::Select(s) => Some(*s),
			Self::Delete(.., s) => Some(*s),
			Self::Yank(s) => Some(*s),
		}
	}

	#[inline]
	pub(super) fn range(&self, cursor: usize, include: bool) -> Option<Range<usize>> {
		self
			.start()
			.map(|s| if s <= cursor { (s, cursor) } else { (cursor, s) })
			.map(|(s, e)| s..e + include as usize)
	}
}
