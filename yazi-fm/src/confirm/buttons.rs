use ratatui::{buffer::Buffer, layout::{Constraint, Rect}, text::Span, widgets::{Paragraph, Widget}};
use yazi_config::THEME;

pub(crate) struct Buttons;

impl Widget for Buttons {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let chunks =
			ratatui::layout::Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(area);

		Paragraph::new(Span::raw(&THEME.confirm.btn_labels[0]).style(THEME.confirm.btn_yes))
			.centered()
			.render(chunks[0], buf);
		Paragraph::new(Span::raw(&THEME.confirm.btn_labels[1]).style(THEME.confirm.btn_no))
			.centered()
			.render(chunks[1], buf);
	}
}
