use ratatui_core::{buffer::Buffer, layout::{Constraint, Rect}, text::Span, widgets::Widget};
use ratatui_widgets::paragraph::Paragraph;
use yazi_config::THEME;

pub(crate) struct Buttons;

impl Widget for Buttons {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let chunks =
			ratatui_core::layout::Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
				.split(area);
		let labels = THEME.confirm.btn_labels.load();

		Paragraph::new(Span::raw(&labels[0]).style(THEME.confirm.btn_yes.get()))
			.centered()
			.render(chunks[0], buf);
		Paragraph::new(Span::raw(&labels[1]).style(THEME.confirm.btn_no.get()))
			.centered()
			.render(chunks[1], buf);
	}
}
