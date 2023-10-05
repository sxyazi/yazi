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
	// fn highlighted_item<'b>(&'b self, file: &'b File) -> Vec<Span> {
	// 	let short = short_path(file.url(), &self.folder.cwd);
	//
	// 	let v = self.is_find.then_some(()).and_then(|_| {
	// 		let finder = self.cx.manager.active().finder()?;
	// 		let (head, body, tail) = finder.explode(short.name)?;
	//
	// 		// TODO: to be configured by THEME?
	// 		let style = Style::new().fg(Color::Rgb(255, 255,
	// 50)).add_modifier(Modifier::ITALIC); 		Some(vec![
	// 			Span::raw(short.prefix.join(head.as_ref()).display().to_string()),
	// 			Span::styled(body, style),
	// 			Span::raw(tail),
	// 		])
	// 	});
	//
	// 	v.unwrap_or_else(|| vec![Span::raw(format!("{}", short))])
	// }
}

impl Widget for Folder {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let x = plugin::Folder { kind: self.kind as u8 }.render(area);
		if x.is_err() {
			info!("{:?}", x);
			return;
		}

		for x in x.unwrap() {
			x.render(buf);
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
