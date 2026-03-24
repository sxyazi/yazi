use std::{boxed::Box, iter, slice::Iter, str::{self, Lines}};

use ratatui::{layout::Alignment, style::Style, text::Line, widgets::Wrap};

use super::wrapper::{LineComposer, WordWrapper};
use crate::ratatui::SpanIter;

type WrappedLines<'lend, 'text> =
	Box<dyn Iterator<Item = (SpanIter<'lend, 'text>, Alignment)> + 'lend>;

pub struct LineIter<'lend, 'text> {
	inner:    LineIterInner<'lend, 'text>,
	tab_size: u8,
}

impl<'lend, 'text> LineIter<'lend, 'text> {
	pub fn source(source: &'text str, tab_size: u8) -> Self {
		Self {
			inner: LineIterInner::Source(if source.is_empty() { "\n" } else { source }.lines()),
			tab_size,
		}
	}

	pub fn parsed(text: &'lend [Line<'text>], tab_size: u8) -> Self {
		if text.is_empty() {
			Self::source("", tab_size)
		} else {
			Self { inner: LineIterInner::Parsed(text.iter()), tab_size }
		}
	}

	pub fn wrapped(mut self, wrap: Wrap, width: u16) -> Self {
		if width == 0 || matches!(self.inner, LineIterInner::Wrapped(_)) {
			return self; // No wrapping needed, return the original iterator
		}

		let lines = Box::new(iter::from_fn(move || self.next_owned()));
		Self {
			inner:    LineIterInner::Wrapped(WordWrapper::new(lines, width, wrap.trim)),
			tab_size: 0,
		}
	}

	pub fn next<'a>(&'a mut self) -> Option<(SpanIter<'a, 'text>, Alignment)> {
		match self.inner {
			LineIterInner::Source { .. } | LineIterInner::Parsed { .. } => self.next_owned(),
			LineIterInner::Wrapped(ref mut wrapper) => {
				let wrapped = wrapper.next_line()?;
				Some((SpanIter::Wrapped(wrapped.graphemes.iter()), wrapped.alignment))
			}
		}
	}

	fn next_owned(&mut self) -> Option<(SpanIter<'lend, 'text>, Alignment)> {
		match &mut self.inner {
			LineIterInner::Source(it) => {
				Some((SpanIter::from_span(it.next()?, Style::new(), self.tab_size), Alignment::Left))
			}
			LineIterInner::Parsed(it) => {
				let line = it.next()?;
				let alignment = line.alignment.unwrap_or(Alignment::Left);
				Some((SpanIter::from_line(&line.spans, line.style, self.tab_size), alignment))
			}
			LineIterInner::Wrapped(_) => {
				unreachable!(); // This branch is handled by next() and should never call next_owned()
			}
		}
	}
}

// -- LineIterInner
enum LineIterInner<'lend, 'text>
where
	'lend: 'text,
{
	Source(Lines<'text>),
	Parsed(Iter<'lend, Line<'text>>),
	Wrapped(WordWrapper<'text, WrappedLines<'lend, 'text>, SpanIter<'lend, 'text>>),
}
