use std::{borrow::Cow, slice::Iter, vec::IntoIter};

use ratatui::{style::Style, text::{Line, Span}};
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

use crate::ratatui::StyledGrapheme;

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
		Line::from_iter(self.map(|g| Span { style: g.style, content: g.symbol.into_owned().into() }))
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
					&& let Some(symbol) = span.next_symbol()
				{
					if symbol == "\t" {
						if *tab_size == 0 {
							continue;
						}

						*pending_tabs = tab_size.saturating_sub(1);
						*pending_style = span.style();
						return Some(StyledGrapheme::new(" ", span.style()));
					}

					return Some(StyledGrapheme::new(symbol, span.style()));
				}

				let span = spans.next()?;
				*current = Some(match span.content {
					Cow::Borrowed(content) => CurrentSpan::Borrowed {
						style:     line_style.patch(span.style),
						graphemes: content.graphemes(true),
					},
					Cow::Owned(content) => CurrentSpan::Owned {
						style:     line_style.patch(span.style),
						graphemes: content.graphemes(true).map(str::to_owned).collect::<Vec<_>>().into_iter(),
					},
				});
			},
		}
	}
}

// --- CurrentSpan
enum CurrentSpan<'text> {
	Borrowed { style: Style, graphemes: Graphemes<'text> },
	Owned { style: Style, graphemes: IntoIter<String> },
}

impl<'text> CurrentSpan<'text> {
	fn style(&self) -> Style {
		match self {
			Self::Borrowed { style, .. } | Self::Owned { style, .. } => *style,
		}
	}

	fn next_symbol(&mut self) -> Option<Cow<'text, str>> {
		match self {
			Self::Borrowed { graphemes, .. } => graphemes.next().map(Cow::Borrowed),
			Self::Owned { graphemes, .. } => graphemes.next().map(Cow::Owned),
		}
	}
}
