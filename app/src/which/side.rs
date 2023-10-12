use config::{keymap::Control, THEME};
use ratatui::{prelude::{Buffer, Rect}, text::{Line, Span}, widgets::{Block, List, ListItem, Padding, Widget}};

pub(super) struct Side<'a> {
	times: usize,
	cands: Vec<&'a Control>,
}

impl<'a> Side<'a> {
	pub(super) fn new(times: usize, cands: Vec<&'a Control>) -> Self { Self { times, cands } }
}

impl Widget for Side<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let items = self
			.cands
			.into_iter()
			.map(|c| {
				let mut spans = Vec::with_capacity(10);

				// Keys
				let keys = c.on[self.times..].iter().map(ToString::to_string).collect::<Vec<_>>();
				spans.push(Span::raw(" ".repeat(10usize.saturating_sub(keys.join("").len()))));
				spans.push(Span::styled(keys[0].clone(), THEME.which.cand.into()));
				spans.extend(
					keys.iter().skip(1).map(|k| Span::styled(k.to_string(), THEME.which.rest.into())),
				);

				// Separator
				spans.push(Span::styled(
					THEME.which.separator.to_string(),
					THEME.which.separator_style.into(),
				));

				// Exec
				spans.push(Span::styled(c.desc_or_exec(), THEME.which.desc.into()));

				ListItem::new(Line::from(spans))
			})
			.collect::<Vec<_>>();

		List::new(items).block(Block::new().padding(Padding::new(0, 1, 1, 1))).render(area, buf);
	}
}
