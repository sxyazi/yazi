use std::{boxed::Box, iter, mem, str::{self, Lines}, vec::IntoIter};

use ratatui::{layout::Alignment, text::Line, widgets::Wrap};

use super::wrapper::{LineComposer, WordWrapper};
use crate::ratatui::SpanIter;

type WrappedLines<'text> = Box<dyn Iterator<Item = (SpanIter<'text, 'text>, Alignment)> + 'text>;

pub struct LineIter<'text> {
	inner:    LineIterInner<'text>,
	tab_size: u8,
}

enum LineIterInner<'text> {
	Source { empty: bool, lines: Lines<'text> },
	Parsed(IntoIter<Line<'text>>),
	Wrapped(WordWrapper<'text, WrappedLines<'text>, SpanIter<'text, 'text>>),
	Unlimited(WrappedLines<'text>),
}

impl<'text> LineIter<'text> {
	pub fn source(source: &'text str, tab_size: u8) -> Self {
		Self {
			inner: LineIterInner::Source { empty: source.is_empty(), lines: source.lines() },
			tab_size,
		}
	}

	pub fn parsed(mut text: Vec<Line<'text>>, tab_size: u8) -> Self {
		if text.is_empty() {
			text.push(Line::from(""));
		}
		Self { inner: LineIterInner::Parsed(text.into_iter()), tab_size }
	}

	pub fn wrapped(mut self, wrap: Wrap, width: u16) -> Self {
		let lines = Box::new(iter::from_fn(move || self.next_owned()));
		Self {
			inner:    if width == 0 {
				LineIterInner::Unlimited(lines)
			} else {
				LineIterInner::Wrapped(WordWrapper::new(lines, width, wrap.trim))
			},
			tab_size: 0,
		}
	}

	pub fn next<'lend>(&'lend mut self) -> Option<(SpanIter<'lend, 'text>, Alignment)> {
		match self.inner {
			LineIterInner::Source { .. } | LineIterInner::Parsed(_) => self.next_owned(),
			LineIterInner::Wrapped(ref mut wrapper) => {
				let wrapped = wrapper.next_line()?;
				Some((SpanIter::Wrapped(wrapped.graphemes.iter()), wrapped.alignment))
			}
			LineIterInner::Unlimited(ref mut lines) => lines.next(),
		}
	}

	fn next_owned(&mut self) -> Option<(SpanIter<'text, 'text>, Alignment)> {
		match &mut self.inner {
			LineIterInner::Source { empty, lines } => {
				let line = if mem::replace(empty, false) {
					Some(Line::from(""))
				} else {
					lines.next().map(Line::from)
				}?;
				let alignment = line.alignment.unwrap_or(Alignment::Left);

				Some((SpanIter::new(line, self.tab_size), alignment))
			}
			LineIterInner::Parsed(lines) => {
				let line = lines.next()?;
				let alignment = line.alignment.unwrap_or(Alignment::Left);

				Some((SpanIter::new(line, self.tab_size), alignment))
			}
			LineIterInner::Wrapped(_) | LineIterInner::Unlimited(_) => {
				unreachable!(); // This branch is handled by next() and should never call next_owned()
			}
		}
	}
}
