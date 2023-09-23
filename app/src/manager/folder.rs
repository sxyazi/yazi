use core::files::File;

use config::{MANAGER, THEME};
use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{List, ListItem, Widget}};
use shared::{short_path, short_path_parts, PathParts};

use crate::Ctx;

pub(super) struct Folder<'a> {
	cx:           &'a Ctx,
	folder:       &'a core::manager::Folder,
	location:     FolderLocation,
	is_selection: bool,
	is_find:      bool,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(super) enum FolderLocation {
	Parent,
	Current,
	Preview,
}

impl<'a> Folder<'a> {
	pub(super) fn new(
		cx: &'a Ctx,
		folder: &'a core::manager::Folder,
		location: FolderLocation,
	) -> Self {
		Self { cx, folder, location, is_selection: false, is_find: false }
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

		// TODO to be configured by THEME?
		let find_style = Style::new().fg(Color::Rgb(255, 255, 50)).add_modifier(Modifier::ITALIC);

		let window = if self.location == FolderLocation::Preview {
			self.folder.window_for(active.preview().skip())
		} else {
			self.folder.window()
		};

		let items: Vec<_> = window
			.iter()
			.enumerate()
			.map(|(i, f)| {
				let is_selected = self.folder.files.is_selected(f.url());
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

				let hovered = matches!(self.folder.hovered, Some(ref h) if h.url() == f.url());
				let style = if self.location == FolderLocation::Preview && hovered {
					THEME.preview.hovered.get()
				} else if hovered {
					THEME.selection.hovered.get()
				} else {
					self.file_style(f)
				};

				let mut spans = Vec::with_capacity(10);

				spans.push(Span::raw(format!(" {} ", Self::icon(f))));
				if let Some((path, (n_prefix, n_hl, n_suffix))) =
					// If this is the current folder,
					(self.location == FolderLocation::Current).then_some(()).and_then(|_| {
							// ... and we are now finding something,
							let finder = active.finder()?;
							// ... and we can get the short path parts,
							let PathParts { path, filename } = short_path_parts(f.url(), &self.folder.cwd)?;
							// ... and we can get the highlight range,
							Some((path, finder.try_render_highlight(filename)?))
						}) {
					// ... then render the highlighted short path.
					spans.push(Span::raw(path.join(n_prefix).display().to_string()));
					spans.push(Span::styled(n_hl, find_style));
					spans.push(Span::raw(n_suffix));
				} else {
					// Otherwise, just render the short path.
					spans.push(Span::raw(short_path(f.url(), &self.folder.cwd)));
				}

				if let Some(link_to) = f.link_to() {
					if MANAGER.show_symlink {
						spans.push(Span::raw(format!(" -> {}", link_to.display())));
					}
				}

				if hovered && self.is_find {
					if let Some(idx) = active
						.finder()
						.filter(|&f| f.has_matched())
						.and_then(|finder| finder.matched_idx(f.url()))
					{
						let len = active.finder().unwrap().matched().len();
						spans.push(Span::styled(
							format!(
								"  [{}/{}]",
								if idx > 99 { ">99".to_string() } else { (idx + 1).to_string() },
								if len > 99 { ">99".to_string() } else { len.to_string() }
							),
							find_style,
						));
					}
				}

				ListItem::new(Line::from(spans)).style(style)
			})
			.collect();

		List::new(items).render(area, buf);
	}
}
