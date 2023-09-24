use core::files::File;

use config::{MANAGER, THEME};
use ratatui::{
	buffer::Buffer,
	layout::Rect,
	style::{Color, Modifier, Style},
	text::{Line, Span},
	widgets::{List, ListItem, Widget},
};
use shared::short_path;

use crate::Ctx;

pub(super) struct Folder<'a> {
	cx: &'a Ctx,
	folder: &'a core::manager::Folder,
	is_preview: bool,
	is_selection: bool,
	is_find: bool,
}

impl<'a> Folder<'a> {
	pub(super) fn new(cx: &'a Ctx, folder: &'a core::manager::Folder) -> Self {
		Self { cx, folder, is_preview: false, is_selection: false, is_find: false }
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
	pub(super) fn with_find(mut self, state: bool) -> Self {
		self.is_find = state;
		self
	}
}

impl<'a> Folder<'a> {
	#[inline]
	fn icon(file: &File) -> &'static str {
		THEME
			.icons
			.iter()
			.find(|x| x.name.match_path(file.url(), Some(file.is_dir())))
			.map(|x| x.display.as_ref())
			.unwrap_or("")
	}

	#[inline]
	fn file_style(&self, file: &File) -> Style {
		let mimetype = &self.cx.manager.mimetype;
		THEME
			.filetypes
			.iter()
			.find(|x| x.matches(file.url(), mimetype.get(file.url()), file.is_dir()))
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

		let items: Vec<_> = window
			.iter()
			.enumerate()
			.map(|(i, file)| {
				let is_selected = self.folder.files.is_selected(file.url());
				if (!self.is_selection && is_selected)
					|| (self.is_selection && mode.pending(self.folder.offset() + i, is_selected))
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

				let hovered =
					matches!(self.folder.hovered, Some(ref hover) if hover.url() == file.url());
			    
				let style = match (self.is_preview, hovered) {
					(true, true) => THEME.preview.hovered.get(),
					(_, true) => THEME.selection.hovered.get(),
					_ => self.file_style(file),
				};

				let mut spans = Vec::with_capacity(10);

				spans.push(Span::raw(format!(" {} ", Self::icon(file))));
				spans.push(Span::raw(short_path(file.url(), &self.folder.cwd)));

				if let Some(link_to) = file.link_to() {
					if MANAGER.show_symlink {
						spans.push(Span::raw(format!(" -> {}", link_to.display())));
					}
				}

				if let Some(idx) = active
					.finder()
					.filter(|&finder| hovered && self.is_find && finder.has_matched())
					.and_then(|finder| finder.matched_idx(file.url()))
				{
					let len = active.finder().unwrap().matched().len();
					let style = Style::new().fg(Color::Rgb(255, 255, 50)).add_modifier(Modifier::ITALIC);
					spans.push(Span::styled(
						format!(
							"  [{}/{}]",
							if idx > 99 { ">99".to_string() } else { (idx + 1).to_string() },
							if len > 99 { ">99".to_string() } else { len.to_string() }
						),
						style,
					));
				}

				ListItem::new(Line::from(spans)).style(style)
			})
			.collect();

		List::new(items).render(area, buf);
	}
}
