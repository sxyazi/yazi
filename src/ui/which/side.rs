use ratatui::{prelude::{Buffer, Rect}, widgets::{List, ListItem, Widget}};

use crate::config::keymap::Control;

pub struct Side<'a> {
	times: usize,
	cands: Vec<(usize, &'a Control)>,
}

impl<'a> Side<'a> {
	pub fn new(times: usize, cands: Vec<(usize, &'a Control)>) -> Self { Self { times, cands } }
}

impl Widget for Side<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let items = self
			.cands
			.into_iter()
			.map(|(_, c)| {
				let s = c.on[self.times..].into_iter().map(ToString::to_string).collect::<String>();
				ListItem::new(format!("{:?}", s))
			})
			.collect::<Vec<_>>();

		List::new(items).render(area, buf);
	}
}
