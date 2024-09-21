use ratatui::{buffer::Buffer, layout::Rect, text::{Line, Span}, widgets::Widget};
use yazi_config::{THEME, keymap::Chord};

pub(super) struct Cand<'a> {
	cand:  &'a Chord,
	times: usize,
}

impl<'a> Cand<'a> {
	pub(super) fn new(cand: &'a Chord, times: usize) -> Self { Self { times, cand } }

	fn keys(&self) -> Vec<String> {
		self.cand.on[self.times..].iter().map(ToString::to_string).collect()
	}
}

impl Widget for Cand<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let keys = self.keys();
		let mut spans = Vec::with_capacity(10);

		// Padding
		spans.push(Span::raw(" ".repeat(10usize.saturating_sub(keys.join("").len()))));

		// First key
		spans.push(Span::styled(keys[0].clone(), THEME.which.cand));

		// Rest keys
		spans.extend(keys.iter().skip(1).map(|k| Span::styled(k, THEME.which.rest)));

		// Separator
		spans.push(Span::styled(&THEME.which.separator, THEME.which.separator_style));

		// Description
		spans.push(Span::styled(self.cand.desc_or_run(), THEME.which.desc));

		Line::from(spans).render(area, buf);
	}
}
