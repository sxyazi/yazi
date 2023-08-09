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
	pub(super) fn new(value: String) -> Self {
		let mut snap = Self {
			value,

			op: Default::default(),

			mode: Default::default(),
			offset: usize::MAX,
			cursor: usize::MAX,
		};
		snap.reset();
		snap
	}

	#[inline]
	pub(super) fn reset(&mut self) {
		self.cursor = self.cursor.min(self.value.chars().count().saturating_sub(self.mode.delta()));
		self.offset =
			self.offset.min(self.cursor.saturating_sub(Self::find_window(&self.rev(), 0).end));
	}

	pub(super) fn insert(&mut self) -> bool {
		if self.mode != InputMode::Normal {
			return false;
		}

		self.op = InputOp::None;
		self.mode = InputMode::Insert;
		true
	}

	#[allow(clippy::if_same_then_else)]
	pub fn visual(&mut self) -> bool {
		if self.mode != InputMode::Normal {
			return false;
		} else if self.value.is_empty() {
			return false;
		}

		self.op = InputOp::Select(self.cursor);
		true
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
	pub(super) fn window(&self) -> Range<usize> { Self::find_window(&self.value, self.offset) }

	#[inline]
	pub(super) fn find_window(s: &str, offset: usize) -> Range<usize> {
		let mut width = 0;
		let v = s
			.chars()
			.enumerate()
			.skip(offset)
			.map_while(|(i, c)| {
				width += c.width().unwrap_or(0);
				if width < /*TODO: hardcode*/ 50 - 2 { Some(i) } else { None }
			})
			.collect::<Vec<_>>();

		if v.is_empty() {
			return 0..0;
		}
		*v.first().unwrap()..v.last().unwrap() + 1
	}
}
