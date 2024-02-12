use std::rc::Rc;

use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Offset, Rect}, style::{Style, Stylize}, widgets::{Block, BorderType, Paragraph, Widget}};
use yazi_core::notify::{Level, Message};

use crate::{widgets::Clear, Ctx};

pub(crate) struct Notify<'a> {
	cx: &'a Ctx,
}

impl<'a> Notify<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }

	fn chunks(area: Rect, messages: &[Message]) -> Rc<[Rect]> {
		let chunks = Layout::horizontal([Constraint::Percentage(100), Constraint::Min(40)]).split(area);

		Layout::vertical(messages.iter().map(|m| Constraint::Length(m.height() as u16)))
			.spacing(1)
			.split(chunks[1])
	}
}

impl<'a> Widget for Notify<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let notify = &self.cx.notify;

		let limit = notify.limit();
		let chunks = Self::chunks(area, &notify.messages[..limit]);

		for (i, m) in notify.messages.iter().enumerate().take(limit) {
			let (icon, style) = match m.level {
				Level::Info => ("", Style::default().green()),
				Level::Warn => ("", Style::default().yellow()),
				Level::Error => ("", Style::default().red()),
			};

			let mut rect = chunks[i]
				.offset(Offset { x: (100 - m.percent) as i32 * chunks[i].width as i32 / 100, y: 0 });
			rect.width = area.width.saturating_sub(rect.x);

			Clear.render(rect, buf);
			Paragraph::new(m.content.as_str())
				.block(
					Block::bordered()
						.border_type(BorderType::Rounded)
						.title(format!("{} {}", icon, m.title))
						.title_style(style)
						.border_style(style),
				)
				.render(rect, buf);
		}
	}
}
