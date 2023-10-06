use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tracing::info;

pub(super) struct Folder {
	kind: FolderKind,
}

pub(super) enum FolderKind {
	Parent  = 0,
	Current = 1,
	Preview = 2,
}

impl Folder {
	pub(super) fn new(kind: FolderKind) -> Self { Self { kind } }
}

impl Folder {
	// 	let short = short_path(file.url(), &self.folder.cwd);
}

impl Widget for Folder {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let x = plugin::Folder { kind: self.kind as u8 }.render(area, buf);
		if x.is_err() {
			info!("{:?}", x);
		}

		// let items: Vec<_> = window
		// 	.iter()
		// 	.enumerate()
		// 	.map(|(i, f)| {
		// 		if let Some(idx) = active
		// 			.finder()
		// 			.filter(|&f| hovered && self.is_find && f.has_matched())
		// 			.and_then(|finder| finder.matched_idx(f.url()))
		// 		{
		// 			let len = active.finder().unwrap().matched().len();
		// 			spans.push(Span::styled(
		// 				format!(
		// 					"  [{}/{}]",
		// 					if idx > 99 { ">99".to_string() } else { (idx + 1).to_string() },
		// 					if len > 99 { ">99".to_string() } else { len.to_string() }
		// 				),
		// 				// TODO: to be configured by THEME?
		// 				Style::new().fg(Color::Rgb(255, 255, 50)).add_modifier(Modifier::ITALIC),
		// 			));
		// 		}
		//
		// 		ListItem::new(Line::from(spans)).style(style)
		// 	})
		// 	.collect();

		// List::new(items).render(area, buf);
	}
}
