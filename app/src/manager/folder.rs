use core::files::File;

use config::{MANAGER, THEME};
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
			.find(|x| x.matches(file.path(), mimetype.get(file.path()), file.meta.is_dir()))
			.map(|x| x.style.get())
			.unwrap_or_else(Style::new)
	}
}

impl<'a> Widget for Folder<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let active = self.cx.manager.active();
		let mode = active.mode();

		let window = if self.is_preview {
			self.folder.window_for(active.preview().skip())
		} else {
			self.folder.window()
		};

		let items = window
			.iter()
			.enumerate()
			.map(|(i, f)| {
				let icon = THEME
					.icons
					.iter()
					.find(|x| x.name.match_path(f.path(), Some(f.meta.is_dir())))
					.map(|x| x.display.as_ref())
					.unwrap_or("");

				let is_selected = self.folder.files.is_selected(f.path());
				if (!self.is_selection && is_selected)
					|| (self.is_selection && mode.pending(i, is_selected))
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

				let hovered = matches!(self.folder.hovered, Some(ref h) if h.path() == f.path());
				let style = if self.is_preview && hovered {
					THEME.preview.hovered.get()
				} else if hovered {
					THEME.selection.hovered.get()
				} else {
					self.file_style(f)
				};

				let mut path = format!(" {icon} {}", readable_path(f.path(), &self.folder.cwd));
				if let Some(ref link_to) = f.link_to {
					if MANAGER.show_symlink {
						path.push_str(&format!(" -> {}", link_to.display()));
					}
				}

				ListItem::new(path).style(style)
			})
			.collect::<Vec<_>>();

		List::new(items).render(area, buf);
	}
}
