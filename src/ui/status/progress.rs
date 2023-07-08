use ratatui::{style::{Color, Style}, widgets::{Gauge, Widget}};

use crate::ui::Ctx;

pub struct Progress<'a> {
	cx: &'a Ctx,
}

impl<'a> Progress<'a> {
	pub fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Progress<'a> {
	fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
		let progress = &self.cx.tasks.progress;
		if progress.0 >= 100 {
			return;
		}

		Gauge::default()
			.gauge_style(Style::default().fg(Color::Yellow))
			.percent(progress.0 as u16)
			.label(format!("{}%, {} left", progress.0, progress.1))
			.use_unicode(true)
			.render(area, buf);
	}
}
