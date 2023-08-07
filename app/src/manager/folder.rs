use core::files::File;

use config::THEME;
use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::{List, ListItem, Widget}};
use shared::readable_path;

use crate::Ctx;

pub(super) struct Folder<'a> {
	cx:           &'a Ctx,
	folder:       &'a core::manager::Folder,
	is_preview:   bool,
	is_selection: bool,
}

impl<'a> Folder<'a> {
	pub(super) fn new(cx: &'a Ctx, folder: &'a core::manager::Folder) -> Self {
		Self { cx, folder, is_preview: false, is_selection: false }
	}

	#[inline]
	pub(super) fn with_preview(mut self, state: bool) -> Self {
		self.is_preview = state;
		self
	}

	#[inline]
	pub(super) fn with_selection(mut self, state: bool) -> Self {
		self.is_selection = state;
		self
	}

	#[inline]
	fn file_style(&self, file: &File) -> Style {
		let mimetype = &self.cx.manager.mimetype;
		THEME
			.filetypes
			.iter()
			.find(|x| x.matches(&file.path, mimetype.get(&file.path).cloned(), file.meta.is_dir()))
			.map(|x| x.style.get())
			.unwrap_or_else(|| Style::new())
	}
}

impl<'a> Widget for Folder<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let window = self.folder.window();
		let mode = self.cx.manager.active().mode();

		let items = window
			.iter()
			.enumerate()
			.map(|(i, (k, v))| {
				let icon = THEME
					.icons
					.iter()
					.find(|x| x.name.match_path(k, Some(v.meta.is_dir())))
					.map(|x| x.display.as_ref())
					.unwrap_or("");

				if (!self.is_selection && v.is_selected)
					|| (self.is_selection && mode.pending(i, v.is_selected))
				{
					buf.set_style(
						Rect { x: area.x.saturating_sub(1), y: i as u16 + 1, width: 1, height: 1 },
						if self.is_selection {
							THEME.marker.selecting.get()
						} else {
							THEME.marker.selected.get()
						},
					);
				}

				let hovered = matches!(self.folder.hovered, Some(ref h) if h.path == *k);
				let style = if self.is_preview && hovered {
					THEME.preview.hovered.get()
				} else if hovered {
					THEME.selection.hovered.get()
				} else {
					self.file_style(v)
				};

				ListItem::new(format!(" {icon} {}", readable_path(k, &self.folder.cwd))).style(style)
			})
			.collect::<Vec<_>>();

		List::new(items).render(area, buf);
	}
}
