use config::keymap::Control;
use ratatui::{prelude::{Buffer, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, List, ListItem, Padding, Widget}};

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
				let mut spans = vec![];

				// Keys
				let keys = c.on[self.times..].iter().map(ToString::to_string).collect::<Vec<_>>();
				spans.push(Span::raw(" ".repeat(10usize.saturating_sub(keys.join("").len()))));
				spans.push(Span::styled(keys[0].clone(), Style::new().fg(Color::LightCyan)));
				spans.extend(
					keys
						.iter()
						.skip(1)
						.map(|k| Span::styled(k.to_string(), Style::new().fg(Color::DarkGray))),
				);

				// Separator
				spans.push(Span::styled("  ".to_string(), Style::new().fg(Color::DarkGray)));

				// Exec
				let exec = c.exec.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ");
				spans.push(Span::styled(exec, Style::new().fg(Color::Magenta)));

				ListItem::new(Line::from(spans))
			})
			.collect::<Vec<_>>();

		List::new(items).block(Block::new().padding(Padding::new(0, 1, 1, 1))).render(area, buf);
	}
}
