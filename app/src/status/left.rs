use core::Ctx;

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

pub(super) struct Left<'a> {
	cx: &'a Ctx,
}

impl<'a> Left<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Left<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		// let folder = self.cx.manager.current();
		// let mode = self.cx.manager.active().mode();
		//
		// // Separator
		// let separator = &THEME.status.separator;
		//
		// // Mode
		// let mut spans = Vec::with_capacity(5);
		// spans.push(Span::styled(&separator.opening, primary.fg()));
		// spans.push(Span::styled(
		// 	format!(" {mode} "),
		// 	primary.bg().fg(**secondary).add_modifier(Modifier::BOLD),
		// ));
		//
		// if let Some(h) = &folder.hovered {
		// 	// Length
		// 	{
		// 		let size = if h.is_dir() { folder.files.size(h.url()) } else { None };
		// 		spans.push(Span::styled(
		// 			format!(" {} ", readable_size(size.unwrap_or(h.length()))),
		// 			body.bg().fg(**primary),
		// 		));
		// 		spans.push(Span::styled(&separator.closing, body.fg()));
		// 	}
		//
		// 	// Filename
		// 	spans.push(Span::raw(format!(" {} ", h.name_display().unwrap())));
		// }
		//
		// Paragraph::new(Line::from(spans)).render(area, buf);
	}
}
