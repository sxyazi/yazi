use ratatui::{buffer::Buffer, layout::{self, Constraint, Offset, Rect}, widgets::{Block, BorderType, Paragraph, Widget, Wrap}};
use yazi_core::notify::Message;

use crate::Ctx;

pub(crate) struct Layout<'a> {
	cx: &'a Ctx,
}

impl<'a> Layout<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }

	pub(crate) fn available(area: Rect) -> Rect {
		let chunks = layout::Layout::horizontal([Constraint::Fill(1), Constraint::Min(80)]).split(area);

		let chunks =
			layout::Layout::vertical([Constraint::Max(1), Constraint::Fill(1)]).split(chunks[1]);

		chunks[1]
	}

	fn tile(area: Rect, messages: &[Message]) -> Vec<Rect> {
		layout::Layout::vertical(
			messages.iter().map(|m| Constraint::Length(m.height(area.width) as u16)),
		)
		.spacing(1)
		.split(area)
		.iter()
		.zip(messages)
		.map(|(&(mut r), m)| {
			if r.width > m.max_width as u16 {
				r.x = r.x.saturating_add(r.width - m.max_width as u16);
				r.width = m.max_width as u16;
			}
			r
		})
		.collect()
	}
}

impl<'a> Widget for Layout<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let notify = &self.cx.notify;

		let available = Self::available(area);
		let limit = notify.limit(available);
		let tile = Self::tile(available, &notify.messages[..limit]);

		for (i, m) in notify.messages.iter().enumerate().take(limit) {
			let mut rect =
				tile[i].offset(Offset { x: (100 - m.percent) as i32 * tile[i].width as i32 / 100, y: 0 });
			rect.width -= rect.x - tile[i].x;

			yazi_plugin::elements::Clear::default().render(rect, buf);
			Paragraph::new(m.content.as_str())
				.wrap(Wrap { trim: false })
				.block(
					Block::bordered()
						.border_type(BorderType::Rounded)
						.title(format!("{} {}", m.level.icon(), m.title))
						.title_style(*m.level.style())
						.border_style(*m.level.style()),
				)
				.render(rect, buf);
		}
	}
}
