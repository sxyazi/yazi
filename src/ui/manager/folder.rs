use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Modifier, Style}, widgets::{List, ListItem, Widget}};

use crate::{config::THEME, core};

pub struct Folder<'a> {
	folder:       &'a core::Folder,
	is_preview:   bool,
	is_selection: bool,
}

impl<'a> Folder<'a> {
	pub fn new(folder: &'a core::Folder) -> Self {
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
			.enumerate()
			.map(|(i, (_, v))| {
				let icon = THEME
					.icons
					.iter()
					.find(|x| x.name.match_path(&v.path, Some(v.meta.is_dir())))
					.map(|x| x.display.as_ref())
					.unwrap_or("");

				let item = ListItem::new(if v.is_selected {
					format!("> {} {}", icon, v.name)
				} else {
					format!("{} {}", icon, v.name)
				});

				let mut style = Style::default();
				if self.is_selection {
					if i == self.folder.rel_cursor() {
						style = style.fg(Color::Black).bg(Color::Red);
					} else if v.is_selected {
						style = style.fg(Color::Red);
					}
				} else if self.is_preview {
					if i == self.folder.rel_cursor() {
						style = style.add_modifier(Modifier::UNDERLINED)
					}
				} else {
					if i == self.folder.rel_cursor() {
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
