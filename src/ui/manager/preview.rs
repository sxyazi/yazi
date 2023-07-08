use std::io::{stdout, Write};

use ansi_to_tui::IntoText;
use ratatui::{buffer::Buffer, layout::Rect, widgets::{Paragraph, Widget}};

use super::Folder;
use crate::{core::{kitty::Kitty, PreviewData}, ui::{Ctx, Term}};

pub struct Preview<'a> {
	cx: &'a Ctx,
}

impl<'a> Preview<'a> {
	pub fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Preview<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		if self.cx.input.visible || self.cx.tasks.visible {
			stdout().write(Kitty::image_hide()).ok();
			return;
		}

		let manager = &self.cx.manager;
		let hovered = if let Some(h) = manager.hovered() {
			h.clone()
		} else {
			stdout().write(Kitty::image_hide()).ok();
			return;
		};

		let preview = manager.active().preview();
		if preview.path != hovered.path {
			return;
		}

		if !matches!(preview.data, PreviewData::Image(_)) {
			stdout().write(Kitty::image_hide()).ok();
		}

		match &preview.data {
			PreviewData::None => {}
			PreviewData::Folder => {
				if let Some(folder) = manager.active().history(&hovered.path) {
					Folder::new(folder).with_preview(true).render(area, buf);
				}
			}
			PreviewData::Text(s) => {
				let p = Paragraph::new(s.as_bytes().into_text().unwrap());
				p.render(area, buf);
			}
			PreviewData::Image(b) => {
				Term::move_to(area.x, area.y).ok();
				stdout().write(b).ok();
			}
		}
	}
}
