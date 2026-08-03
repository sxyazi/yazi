use std::slice::Iter;

use ratatui_core::{layout::Alignment, style::Style, text::{Line, Text}};
use ratatui_widgets::paragraph::Wrap;

use super::{LineComposer, SpanIter, WordWrapper};

type WrappedLines<'lend, 'text> =
	WordWrapper<'text, TextLines<'lend, 'text>, SpanIter<'lend, 'text>>;

pub struct TextIter<'lend, 'text>
where
	'lend: 'text,
{
	inner: WrappedLines<'lend, 'text>,
}

impl<'lend, 'text> TextIter<'lend, 'text>
where
	'lend: 'text,
{
	pub fn new(text: &'lend Text<'text>, wrap: Wrap, width: u16) -> Self {
		let lines = TextLines {
			lines:     text.lines.iter(),
			style:     text.style,
			alignment: text.alignment.unwrap_or(Alignment::Left),
		};
		Self { inner: WordWrapper::new(lines, width, wrap.trim) }
	}

	pub fn next<'a>(&'a mut self) -> Option<(SpanIter<'a, 'text>, Alignment)> {
		let line = self.inner.next_line()?;
		Some((SpanIter::Wrapped(line.graphemes.iter()), line.alignment))
	}
}

// --- TextLines
struct TextLines<'lend, 'text> {
	lines:     Iter<'lend, Line<'text>>,
	style:     Style,
	alignment: Alignment,
}

impl<'lend, 'text> Iterator for TextLines<'lend, 'text>
where
	'lend: 'text,
{
	type Item = (SpanIter<'lend, 'text>, Alignment);

	fn next(&mut self) -> Option<Self::Item> {
		let line = self.lines.next()?;
		Some((
			SpanIter::from_line(&line.spans, self.style.patch(line.style), 0),
			line.alignment.unwrap_or(self.alignment),
		))
	}
}
