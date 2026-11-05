use ratatui_core::{buffer::Buffer, layout::Rect, text::{Line, Span}, widgets::Widget};
use yazi_config::{THEME, keymap::Chord};
use yazi_shim::CompactString;

pub(super) struct Cand<'a> {
	cand:  &'a Chord,
	times: usize,
}

impl<'a> Cand<'a> {
	pub(super) fn new(cand: &'a Chord, times: usize) -> Self { Self { times, cand } }

	fn keys(&self) -> Vec<CompactString> {
		self.cand.on[self.times..].iter().map(CompactString::from_display).collect()
	}
}

impl Widget for Cand<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let keys = self.keys();
		let mut spans = Vec::with_capacity(10);
		let separator = &**THEME.which.separator.load();

		// Padding
		spans
			.push(Span::raw(&"          "[..10usize.saturating_sub(keys.iter().map(|k| k.len()).sum())]));

		// First key
		spans.push(Span::styled(&keys[0], THEME.which.cand.get()));

		// Rest keys
		spans.extend(keys.iter().skip(1).map(|k| Span::styled(k, THEME.which.rest.get())));

		// Separator
		spans.push(Span::styled(separator, THEME.which.separator_style.get()));

		// Description
		spans.push(Span::styled(self.cand.desc_or_run(), THEME.which.desc.get()));

		Line::from(spans).render(area, buf);
	}
}
