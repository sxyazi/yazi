use core::input::InputMode;
use std::ops::Range;

use ansi_to_tui::IntoText;
use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style}, text::{Line, Text}, widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget}};
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
		let area = self.cx.area(&input.position);

		let value = if let Ok(v) = input.value_pretty() {
			v.into_text().unwrap()
		} else {
			Text::from(input.value())
		};

		Clear.render(area, buf);
		Paragraph::new(value)
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

		if let Some(Range { start, end }) = input.selected() {
			buf.set_style(
				Rect { x: area.x + 1 + start, y: area.y + 1, width: end - start, height: 1 },
				Style::new().bg(Color::Rgb(72, 77, 102)),
			)
		}

		let _ = match input.mode() {
			InputMode::Insert => Term::set_cursor_bar(),
			_ => Term::set_cursor_block(),
		};
	}
}
