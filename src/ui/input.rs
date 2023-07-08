use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style}, text::Line, widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget}};

use super::{Ctx, Term};
use crate::core::InputMode;

pub struct Input<'a> {
	cx: &'a Ctx,
}

impl<'a> Input<'a> {
	pub fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Input<'a> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let input = &self.cx.input;
		let area = input.area();

		Clear.render(area, buf);
		Paragraph::new(input.value())
			.block(
				Block::default()
					.borders(Borders::ALL)
					.border_style(Style::default().fg(Color::Blue))
					.border_type(BorderType::Rounded)
					.title({
						let mut line = Line::from(input.title());
						line.patch_style(Style::default().fg(Color::White));
						line
					}),
			)
			.style(Style::default().fg(Color::White))
			.render(area, buf);

		let _ = match input.mode() {
			InputMode::Insert => Term::set_cursor_bar(),
			_ => Term::set_cursor_block(),
		};
	}
}
