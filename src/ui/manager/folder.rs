use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Modifier, Style}, widgets::{List, ListItem, Widget}};

use crate::{config::THEME, core, misc::readable_path};

pub struct Folder<'a> {
	folder:       &'a core::manager::Folder,
	is_preview:   bool,
	is_selection: bool,
}

impl<'a> Folder<'a> {
	pub fn new(folder: &'a core::manager::Folder) -> Self {
		Self { folder, is_preview: false, is_selection: true }
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
		let page = self.folder.paginate();

		let items = page
			.iter()
			.enumerate()
			.map(|(i, (k, v))| {
				let icon = THEME
					.icons
					.iter()
					.find(|x| x.name.match_path(k, Some(v.meta.is_dir())))
					.map(|x| x.display.as_ref())
					.unwrap_or("");

				if v.is_selected {
					buf.set_style(
						Rect { x: area.x.saturating_sub(1), y: i as u16 + 1, width: 1, height: 1 },
						Style::default().fg(Color::Red).bg(Color::Red),
					);
				}

				let name = if self.is_selection && v.is_selected {
					format!("  {} {}", icon, readable_path(k, &self.folder.cwd))
				} else {
					format!(" {} {}", icon, readable_path(k, &self.folder.cwd))
				};

				let hovered = matches!(self.folder.hovered, Some(ref h) if h.path == *k);
				let mut style = Style::default();
				if self.is_preview {
					if hovered {
						style = style.add_modifier(Modifier::UNDERLINED)
					}
				} else {
					if hovered {
						style = style.fg(Color::Black).bg(Color::Yellow);
					}
				}

				ListItem::new(name).style(style)
			})
			.collect::<Vec<_>>();

		List::new(items).render(area, buf);
	}
}
