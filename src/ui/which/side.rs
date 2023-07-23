use ratatui::{prelude::{Buffer, Rect}, style::{Color, Style, Stylize}, text::{Line, Span}, widgets::{Block, List, ListItem, Padding, Widget}};

use crate::config::keymap::Control;

pub struct Side<'a> {
	times: usize,
	cands: Vec<&'a Control>,
}

impl<'a> Side<'a> {
	pub fn new(times: usize, cands: Vec<&'a Control>) -> Self { Self { times, cands } }
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
				spans.push(Span::styled(keys[0].clone(), Style::default().fg(Color::LightCyan)));
				spans.extend(
					keys
						.iter()
						.skip(1)
						.map(|k| Span::styled(k.to_string(), Style::default().fg(Color::DarkGray))),
				);

				// Separator
				spans.push(Span::styled("  ".to_string(), Style::default().fg(Color::DarkGray)));

				// Exec
				let exec = c.exec.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ");
				spans.push(Span::styled(exec, Style::default().fg(Color::Magenta)));

				ListItem::new(Line::from(spans))
			})
			.collect::<Vec<_>>();

		List::new(items)
			.block(Block::new().padding(Padding::new(0, 1, 1, 1)))
			.bg(Color::Rgb(47, 51, 73))
			.render(area, buf);
	}
}
