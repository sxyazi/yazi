use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Modifier, Style}, widgets::{List, ListItem, Widget}};

use crate::{config::THEME, core, misc::readable_path};

pub struct Folder<'a> {
	folder:       &'a core::manager::Folder,
	is_preview:   bool,
	is_selection: bool,
}

impl<'a> Folder<'a> {
	pub fn new(folder: &'a core::manager::Folder) -> Self {
		Self { folder, is_preview: false, is_selection: false }
	}

	#[inline]
	pub fn with_preview(mut self, state: bool) -> Self {
		self.is_preview = state;
		self
	}

	#[inline]
	pub fn with_selection(mut self, state: bool) -> Self {
		self.is_selection = state;
		self
	}
}

impl<'a> Widget for Folder<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let items = self
			.folder
			.paginate()
			.iter()
			.map(|(k, v)| {
				let icon = THEME
					.icons
					.iter()
					.find(|x| x.name.match_path(k, Some(v.meta.is_dir())))
					.map(|x| x.display.as_ref())
					.unwrap_or("");

				let name = readable_path(k, &self.folder.cwd);
				let item = ListItem::new(if v.is_selected {
					format!("> {} {}", icon, name)
				} else {
					format!("{} {}", icon, name)
				});
				let hovered = matches!(self.folder.hovered, Some(ref h) if h.path == *k);

				let mut style = Style::default();
				if self.is_selection {
					if hovered {
						style = style.fg(Color::Black).bg(Color::Red);
					} else if v.is_selected {
						style = style.fg(Color::Red);
					}
				} else if self.is_preview {
					if hovered {
						style = style.add_modifier(Modifier::UNDERLINED)
					}
				} else {
					if hovered {
						style = style.fg(Color::Black).bg(Color::Yellow);
					} else if v.is_selected {
						style = style.fg(Color::Red);
					}
				}

				item.style(style)
			})
			.collect::<Vec<_>>();

		List::new(items).render(area, buf);
	}
}
