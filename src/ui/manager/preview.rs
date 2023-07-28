use ansi_to_tui::IntoText;
use ratatui::{buffer::Buffer, layout::Rect, widgets::{Paragraph, Widget}};

use super::Folder;
use crate::{core::manager::PreviewData, ui::Ctx};

pub struct Preview<'a> {
	cx: &'a Ctx,
}

impl<'a> Preview<'a> {
	pub fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Preview<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		// TODO: image
		// if self.cx.input.visible || self.cx.select.visible || self.cx.tasks.visible {
		// }

		let manager = &self.cx.manager;
		let hovered = if let Some(h) = manager.hovered() {
			h.path()
		} else {
			return;
		};

		let preview = manager.active().preview();
		if preview.path != hovered {
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
		}
	}
}
