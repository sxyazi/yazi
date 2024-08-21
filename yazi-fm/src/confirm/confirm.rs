use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Margin, Rect}, text::Line, widgets::{Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget, Wrap}};
use yazi_config::THEME;

use crate::Ctx;

pub(crate) struct Confirm<'a> {
	cx: &'a Ctx,
}

impl<'a> Confirm<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}
impl<'a> Widget for Confirm<'a> {
	fn render(self, _win: Rect, buf: &mut Buffer) {
		let confirm = &self.cx.confirm;
		let area = self.cx.area(&confirm.position);

		yazi_plugin::elements::Clear::default().render(area, buf);

		Block::bordered()
			.border_type(BorderType::Rounded)
			.border_style(THEME.input.border)
			.title(Line::styled(&confirm.title, THEME.input.title))
			.render(area, buf);

		let popup_layout =
			Layout::vertical(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
				.vertical_margin(1)
				.horizontal_margin(2)
				.split(area);

		let button_layout = Layout::horizontal(vec![
			Constraint::Percentage(10),
			Constraint::Percentage(30),
			Constraint::Percentage(20),
			Constraint::Percentage(30),
			Constraint::Percentage(10),
		])
		.vertical_margin(1)
		.split(popup_layout[1]);

		Paragraph::new(confirm.content.lines().map(Line::from).collect::<Vec<Line>>())
			.block(Block::bordered().border_type(BorderType::Rounded).border_style(THEME.input.border))
			.scroll((confirm.offset as u16, 0))
			.wrap(Wrap { trim: false })
			.render(popup_layout[0], buf);

		const BORDER_SIZE: usize = 2;
		if confirm.lines > popup_layout[0].as_size().height as usize - BORDER_SIZE {
			let mut scrollbar_state =
				ScrollbarState::new(confirm.content.lines().collect::<Vec<&str>>().len())
					.position(confirm.offset);

			Scrollbar::new(ScrollbarOrientation::VerticalRight).render(
				popup_layout[0].inner(Margin { vertical: 1, horizontal: 0 }),
				buf,
				&mut scrollbar_state,
			);
		}

		Paragraph::new("[Y]es").block(Block::bordered()).centered().render(button_layout[1], buf);
		Paragraph::new("(N)o").block(Block::bordered()).centered().render(button_layout[3], buf);
	}
}
