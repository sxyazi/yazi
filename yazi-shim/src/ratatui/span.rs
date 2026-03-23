use std::{borrow::Cow, slice::Iter, vec::IntoIter};

use ratatui::{style::Style, text::{Line, Span, StyledGrapheme}};
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

#[allow(private_interfaces)]
pub enum SpanIter<'lend, 'text> {
	Line {
		spans:         IntoIter<Span<'text>>,
		line_style:    Style,
		current:       Option<CurrentSpan<'text>>,
		tab_size:      u8,
		pending_tabs:  u8,
		pending_style: Style,
	},
	Wrapped(Iter<'lend, StyledGrapheme<'text>>),
}

struct CurrentSpan<'text> {
	style:     Style,
	graphemes: Graphemes<'text>,
}

impl<'lend, 'text> SpanIter<'lend, 'text> {
	pub(super) fn new(line: Line<'text>, tab_size: u8) -> Self {
		Self::Line {
			spans: line.spans.into_iter(),
			line_style: line.style,
			current: None,
			tab_size,
			pending_tabs: 0,
			pending_style: Style::default(),
		}
	}

	pub fn into_static_line(self) -> Line<'static> {
		Line::from_iter(self.map(|g| Span { style: g.style, content: g.symbol.to_owned().into() }))
	}
}

impl<'lend, 'text> Iterator for SpanIter<'lend, 'text> {
	type Item = StyledGrapheme<'text>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Wrapped(inner) => inner.next().cloned(),
			Self::Line { spans, line_style, current, tab_size, pending_tabs, pending_style } => loop {
				if *pending_tabs > 0 {
					*pending_tabs -= 1;
					return Some(StyledGrapheme::new(" ", *pending_style));
				}

				if let Some(span) = current
					&& let Some(symbol) = span.graphemes.next()
				{
					if symbol == "\t" {
						if *tab_size == 0 {
							continue;
						}

						*pending_tabs = tab_size.saturating_sub(1);
						*pending_style = span.style;
						return Some(StyledGrapheme::new(" ", span.style));
					}

					return Some(StyledGrapheme::new(symbol, span.style));
				}

				let span = spans.next()?;
				let content = match &span.content {
					Cow::Borrowed(s) => *s,
					// Owned content cannot be safely projected to 'text: the String lives
					// only as long as `span` (a local variable), so grapheme references
					// would dangle after this loop iteration.  Skip the span rather than
					// panic.  In normal usage every Span contains Cow::Borrowed text, so
					// this branch is unreachable when the code is used correctly.
					Cow::Owned(_) => continue,
				};
				*current = Some(CurrentSpan {
					style:     line_style.patch(span.style),
					graphemes: content.graphemes(true),
				});
			},
		}
	}
}
