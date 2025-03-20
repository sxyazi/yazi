use std::ops::Range;

use unicode_width::UnicodeWidthChar;

use super::{InputMode, InputOp};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputSnap {
	pub value: String,

	pub op: InputOp,

	pub mode:   InputMode,
	pub offset: usize,
	pub cursor: usize,
}

impl InputSnap {
	pub(super) fn new(value: String, limit: usize) -> Self {
		let mut snap = Self {
			value,

			op: Default::default(),

			mode: Default::default(),
			offset: usize::MAX,
			cursor: usize::MAX,
		};
		snap.resize(limit);
		snap
	}

	#[inline]
	pub(super) fn resize(&mut self, limit: usize) {
		let range = Self::find_window(self.value.chars().rev(), 0, limit);
		self.cursor = self.cursor.min(self.count().saturating_sub(self.mode.delta()));
		self.offset = self.offset.min(self.cursor.saturating_sub(range.end));
	}
}

impl InputSnap {
	#[inline]
	pub(super) fn len(&self) -> usize { self.value.len() }

	#[inline]
	pub(super) fn count(&self) -> usize { self.value.chars().count() }

	#[inline]
	pub(super) fn idx(&self, n: usize) -> Option<usize> {
		self
			.value
			.char_indices()
			.nth(n)
			.map(|(i, _)| i)
			.or_else(|| if n == self.count() { Some(self.len()) } else { None })
	}

	#[inline]
	pub(super) fn slice(&self, range: Range<usize>) -> &str {
		let (s, e) = (self.idx(range.start), self.idx(range.end));
		&self.value[s.unwrap()..e.unwrap()]
	}

	#[inline]
	pub(super) fn window(&self, limit: usize) -> Range<usize> {
		Self::find_window(self.value.chars(), self.offset, limit)
	}

	#[inline]
	pub(super) fn find_window<T>(it: T, offset: usize, limit: usize) -> Range<usize>
	where
		T: Iterator<Item = char>,
	{
		let mut width = 0;
		let mut range = None;

		for (i, c) in it.enumerate().skip(offset) {
			width += c.width().unwrap_or(0);
			if width > limit {
				break;
			}
			match range {
				None => range = Some(i..i + 1),
				Some(ref mut r) => r.end = i + 1,
			}
		}
		range.unwrap_or(0..0)
	}
}
