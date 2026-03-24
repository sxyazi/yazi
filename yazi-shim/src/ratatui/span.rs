use std::slice::Iter;

use ratatui::{style::Style, text::{Line, Span, StyledGrapheme}};
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

#[allow(private_interfaces)]
pub enum SpanIter<'lend, 'text> {
	Span {
		style:        Style,
		graphemes:    Graphemes<'text>,
		tab_size:     u8,
		pending_tabs: u8,
	},
	Line {
		spans:         Iter<'lend, Span<'text>>,
		line_style:    Style,
		current:       Option<CurrentSpan<'text>>,
		tab_size:      u8,
		pending_tabs:  u8,
		pending_style: Style,
	},
	Wrapped(Iter<'lend, StyledGrapheme<'text>>),
}

impl<'lend, 'text> SpanIter<'lend, 'text> {
	pub(super) fn from_span(source: &'text str, style: Style, tab_size: u8) -> Self {
		Self::Span { style, graphemes: source.graphemes(true), tab_size, pending_tabs: 0 }
	}

	pub(super) fn from_line(spans: &'lend [Span<'text>], line_style: Style, tab_size: u8) -> Self {
		Self::Line {
			spans: spans.iter(),
			line_style,
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

impl<'lend, 'text> Iterator for SpanIter<'lend, 'text>
where
	'lend: 'text,
{
	type Item = StyledGrapheme<'text>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Span { style, graphemes, tab_size, pending_tabs } => {
				if *pending_tabs > 0 {
					*pending_tabs -= 1;
					return Some(StyledGrapheme::new(" ", *style));
				}

				loop {
					let symbol = graphemes.next()?;
					if symbol != "\t" {
						return Some(StyledGrapheme::new(symbol, *style));
					} else if let Some(n) = tab_size.checked_sub(1) {
						*pending_tabs = n;
						return Some(StyledGrapheme::new(" ", *style));
					}
				}
			}
			Self::Line { spans, line_style, current, tab_size, pending_tabs, pending_style } => loop {
				if *pending_tabs > 0 {
					*pending_tabs -= 1;
					return Some(StyledGrapheme::new(" ", *pending_style));
				}

				if let Some(span) = current
					&& let Some(symbol) = span.graphemes.next()
				{
					if symbol != "\t" {
						return Some(StyledGrapheme::new(symbol, span.style));
					} else if let Some(n) = tab_size.checked_sub(1) {
						*pending_tabs = n;
						*pending_style = span.style;
						return Some(StyledGrapheme::new(" ", span.style));
					}
					continue;
				}

				let span = spans.next()?;
				*current = Some(CurrentSpan {
					style:     line_style.patch(span.style),
					graphemes: span.content.graphemes(true),
				});
			},
			Self::Wrapped(inner) => inner.next().cloned(),
		}
	}
}

// --- CurrentSpan
struct CurrentSpan<'text> {
	style:     Style,
	graphemes: Graphemes<'text>,
}
