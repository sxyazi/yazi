use core::{manager::PreviewData, Ctx};

use ansi_to_tui::IntoText;
use ratatui::{buffer::Buffer, layout::Rect, widgets::{Paragraph, Widget}};

use super::Folder;

pub(super) struct Preview<'a> {
	cx: &'a Ctx,
}

impl<'a> Preview<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Preview<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let manager = &self.cx.manager;
		let Some(hovered) = manager.hovered().map(|h| h.url()) else {
			return;
		};

		let preview = manager.active().preview();
		if !preview.same_path(hovered) {
			return;
		}

		match &preview.lock.as_ref().unwrap().data {
			PreviewData::Folder => {
				Folder::Preview.render(area, buf);
			}
			PreviewData::Text(s) => {
				let p = Paragraph::new(s.as_bytes().into_text().unwrap());
				p.render(area, buf);
			}
			PreviewData::Image => {}
		}
	}
}
