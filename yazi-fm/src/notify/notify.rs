use ratatui::{buffer::Buffer, layout::{self, Constraint, Offset, Rect}, widgets::{Block, BorderType, Paragraph, Widget, Wrap}};
use yazi_core::notify::Message;

use crate::Ctx;

pub(crate) struct Notify<'a> {
	cx: &'a Ctx,
}

impl<'a> Notify<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }

	pub(crate) fn available(area: Rect) -> Rect {
		let chunks = layout::Layout::horizontal([Constraint::Fill(1), Constraint::Min(80)]).split(area);

		let chunks =
			layout::Layout::vertical([Constraint::Fill(1), Constraint::Max(1)]).split(chunks[1]);

		chunks[0]
	}

	fn tiles<'m>(area: Rect, messages: impl Iterator<Item = &'m Message> + Clone) -> Vec<Rect> {
		layout::Layout::vertical(
			[Constraint::Fill(1)]
				.into_iter()
				.chain(messages.clone().map(|m| Constraint::Length(m.height(area.width) as u16))),
		)
		.spacing(1)
		.split(area)
		.iter()
		.skip(1)
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

impl Widget for Notify<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let notify = &self.cx.notify;
		let available = Self::available(area);

		let messages = notify.messages.iter().take(notify.limit(available)).rev();
		let tiles = Self::tiles(available, messages.clone());

		for (i, m) in messages.enumerate() {
			let mut rect =
				tiles[i].offset(Offset { x: (100 - m.percent) as i32 * tiles[i].width as i32 / 100, y: 0 });
			rect.width -= rect.x - tiles[i].x;

			yazi_plugin::elements::Clear::default().render(rect, buf);
			Paragraph::new(m.content.as_str())
				.wrap(Wrap { trim: false })
				.block(
					Block::bordered()
						.border_type(BorderType::Rounded)
						.title(format!("{} {}", m.level.icon(), m.title))
						.title_style(m.level.style())
						.border_style(m.level.style()),
				)
				.render(rect, buf);
		}
	}
}
