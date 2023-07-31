use config::THEME;
use ratatui::{buffer::Buffer, layout::Rect, text::Span, widgets::{Gauge, Widget}};

use crate::Ctx;

pub(super) struct Progress<'a> {
	cx: &'a Ctx,
}

impl<'a> Progress<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Progress<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let progress = &self.cx.tasks.progress;
		if progress.0 >= 100 {
			return;
		}

		Gauge::default()
			.gauge_style(THEME.progress.gauge.get())
			.percent(progress.0 as u16)
			.label(Span::styled(
				format!("{:>3}%, {} left", progress.0, progress.1),
				THEME.progress.label.get(),
			))
			.render(area, buf);
	}
}
