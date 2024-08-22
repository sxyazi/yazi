use ratatui::{buffer::Buffer, layout::{Constraint, Rect}, style::Stylize, text::Span, widgets::{Paragraph, Widget}};

pub(crate) struct Buttons;

impl Widget for Buttons {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let chunks =
			ratatui::layout::Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(area);

		Paragraph::new(Span::raw("  [Y]es  ").reversed()).centered().render(chunks[0], buf);
		Paragraph::new(Span::raw("  (N)o  ")).centered().render(chunks[1], buf);
	}
}
