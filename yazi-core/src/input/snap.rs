use std::ops::Range;

use unicode_width::UnicodeWidthChar;

use super::{InputMode, InputOp};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct InputSnap {
	pub(super) value: String,

	pub(super) op: InputOp,

	pub(super) mode:   InputMode,
	pub(super) offset: usize,
	pub(super) cursor: usize,
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
		snap.reset(limit);
		snap
	}

	#[inline]
	pub(super) fn reset(&mut self, limit: usize) {
		self.cursor = self.cursor.min(self.value.chars().count().saturating_sub(self.mode.delta()));
		self.offset =
			self.offset.min(self.cursor.saturating_sub(Self::find_window(&self.rev(), 0, limit).end));
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
	pub(super) fn rev(&self) -> String { self.value.chars().rev().collect::<String>() }

	#[inline]
	pub(super) fn window(&self, limit: usize) -> Range<usize> {
		Self::find_window(&self.value, self.offset, limit)
	}

	#[inline]
	pub(super) fn find_window(s: &str, offset: usize, limit: usize) -> Range<usize> {
		let mut width = 0;
		let v: Vec<_> = s
			.chars()
			.enumerate()
			.skip(offset)
			.map_while(|(i, c)| {
				width += c.width().unwrap_or(0);
				if width < limit { Some(i) } else { None }
			})
			.collect();

		if v.is_empty() {
			return 0..0;
		}
		*v.first().unwrap()..v.last().unwrap() + 1
	}
}
