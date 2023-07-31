use core::input::InputMode;

use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style}, text::Line, widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget}};
use shared::Term;

use crate::Ctx;

pub(crate) struct Input<'a> {
	cx: &'a Ctx,
}

impl<'a> Input<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Input<'a> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let input = &self.cx.input;
		let area = input.area();

		Clear.render(area, buf);
		Paragraph::new(input.value())
			.block(
				Block::new()
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded)
					.border_style(Style::new().fg(Color::Blue))
					.title({
						let mut line = Line::from(input.title());
						line.patch_style(Style::new().fg(Color::White));
						line
					}),
			)
			.style(Style::new().fg(Color::White))
			.render(area, buf);

		if let Some(selected) = input.selected() {
			buf.set_style(selected, Style::new().bg(Color::Rgb(72, 77, 102)))
		}

		let _ = match input.mode() {
			InputMode::Insert => Term::set_cursor_bar(),
			_ => Term::set_cursor_block(),
		};
	}
}
