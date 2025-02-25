use std::ops::Range;

use anyhow::{Result, bail};
use ratatui::{buffer::Buffer, layout::Rect, text::Line, widgets::{Block, BorderType, Paragraph, Widget}};
use syntect::easy::HighlightLines;
use yazi_config::{PREVIEW, THEME};
use yazi_core::input::InputMode;
use yazi_plugin::external::Highlighter;

use crate::{Ctx, Term};

pub(crate) struct Input<'a> {
	cx: &'a Ctx,
}

impl<'a> Input<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }

	fn highlighted_value(&self) -> Result<Line<'static>> {
		if !self.cx.input.highlight {
			bail!("Highlighting is disabled");
		}

		let (theme, syntaxes) = Highlighter::init();
		if let Some(syntax) = syntaxes.find_syntax_by_name("Bourne Again Shell (bash)") {
			let mut h = HighlightLines::new(syntax, theme);
			let regions = h.highlight_line(self.cx.input.value(), syntaxes)?;
			return Ok(Highlighter::to_line_widget(regions, &PREVIEW.indent()));
		}
		bail!("Failed to find syntax")
	}
}

impl Widget for Input<'_> {
	fn render(self, win: Rect, buf: &mut Buffer) {
		let input = &self.cx.input;
		let area = self.cx.mgr.area(input.position);

		yazi_plugin::elements::Clear::default().render(area, buf);
		Paragraph::new(self.highlighted_value().unwrap_or_else(|_| Line::from(input.value())))
			.block(
				Block::bordered()
					.border_type(BorderType::Rounded)
					.border_style(THEME.input.border)
					.title(Line::styled(&input.title, THEME.input.title)),
			)
			.style(THEME.input.value)
			.render(area, buf);

		if let Some(Range { start, end }) = input.selected() {
			let x = win.width.min(area.x + 1 + start);
			let y = win.height.min(area.y + 1);

			buf.set_style(
				Rect { x, y, width: (end - start).min(win.width - x), height: 1.min(win.height - y) },
				THEME.input.selected,
			)
		}

		_ = match input.mode() {
			InputMode::Insert => Term::set_cursor_bar(),
			InputMode::Replace => Term::set_cursor_underscore(),
			_ => Term::set_cursor_block(),
		};
	}
}
