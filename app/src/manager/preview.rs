use core::manager::{PreviewData, PREVIEW_BORDER};

use ansi_to_tui::IntoText;
use ratatui::{buffer::Buffer, layout::Rect, widgets::{Clear, Paragraph, Widget}};

use super::Folder;
use crate::Ctx;

pub(super) struct Preview<'a> {
	cx: &'a Ctx,
}

impl<'a> Preview<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Preview<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		Clear.render(
			Rect {
				x:      area.x,
				y:      area.y,
				width:  area.width + PREVIEW_BORDER / 2,
				height: area.height,
			},
			buf,
		);

		let manager = &self.cx.manager;
		let hovered = if let Some(h) = manager.hovered() {
			h.path()
		} else {
			return;
		};

		let preview = manager.active().preview();
		if !preview.same_path(&hovered) {
			return;
		}

		match &preview.data {
			PreviewData::None => {}
			PreviewData::Folder => {
				if let Some(folder) = manager.active().history(&hovered) {
					Folder::new(self.cx, folder).with_preview(true).render(area, buf);
				}
			}
			PreviewData::Text(s) => {
				let p = Paragraph::new(s.as_bytes().into_text().unwrap());
				p.render(area, buf);
			}
			PreviewData::Image => {}
		}
	}
}
