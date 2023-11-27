use std::ops::Range;

use ansi_to_tui::IntoText;
use ratatui::{buffer::Buffer, layout::Rect, text::{Line, Text}, widgets::{Block, BorderType, Borders, Paragraph, Widget}};
use yazi_config::THEME;
use yazi_core::{input::InputMode, Ctx};
use yazi_shared::term::Term;

use crate::widgets;

pub(crate) struct Input<'a> {
	cx: &'a Ctx,
}

impl<'a> Input<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Input<'a> {
	fn render(self, win: Rect, buf: &mut Buffer) {
		let input = &self.cx.input;
		let area = self.cx.area(&input.position);

		let value = if let Ok(v) = input.value_pretty() {
			v.into_text().unwrap()
		} else {
			Text::from(input.value())
		};

		widgets::Clear.render(area, buf);
		Paragraph::new(value)
			.block(
				Block::new()
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded)
					.border_style(THEME.input.border.into())
					.title({
						let mut line = Line::from(input.title.as_str());
						line.patch_style(THEME.input.title.into());
						line
					}),
			)
			.style(THEME.input.value.into())
			.render(area, buf);

		if let Some(Range { start, end }) = input.selected() {
			let x = win.width.min(area.x + 1 + start);
			let y = win.height.min(area.y + 1);

			buf.set_style(
				Rect { x, y, width: (end - start).min(win.width - x), height: 1.min(win.height - y) },
				THEME.input.selected.into(),
			)
		}

		_ = match input.mode() {
			InputMode::Insert => Term::set_cursor_bar(),
			_ => Term::set_cursor_block(),
		};
	}
}
