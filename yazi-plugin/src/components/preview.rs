use ansi_to_tui::IntoText;
use ratatui::{buffer::Buffer, layout::Rect, widgets::{Paragraph, Widget}};
use yazi_core::Ctx;
use yazi_shared::event::PreviewData;

use super::Folder;

pub(super) struct Preview<'a> {
	cx: &'a Ctx,
}

impl<'a> Preview<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Preview<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let Some(ref lock) = self.cx.manager.active().preview.lock else {
			return;
		};

		match &lock.data {
			PreviewData::Folder => {
				Folder::preview(self.cx).render(area, buf);
			}
			PreviewData::Text(s) => {
				let p = Paragraph::new(s.as_bytes().into_text().unwrap());
				p.render(area, buf);
			}
			PreviewData::Image => {}
		}
	}
}
