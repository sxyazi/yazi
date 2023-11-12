use std::io::{stdout, BufWriter, Write};

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use yazi_shared::Term;

pub(crate) struct Clear;

impl Widget for Clear {
	fn render(self, area: Rect, buf: &mut Buffer) {
		// let stdout = BufWriter::new(stdout().lock());
		// let s = " ".repeat(area.width as usize);
		// _ = Term::move_lock(stdout, (0, 0), |stdout| {
		// 	for y in area.top()..area.bottom() {
		// 		Term::move_to(stdout, area.x, y)?;
		// 		stdout.write_all(s.as_bytes())?;
		// 	}
		// 	Ok(())
		// });

		ratatui::widgets::Clear.render(area, buf);
	}
}
