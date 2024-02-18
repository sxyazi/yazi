use crossterm::terminal::WindowSize;
use ratatui::layout::Rect;
use yazi_shared::{event::Cmd, term::Term};

use crate::app::App;

impl App {
	pub(crate) fn update_notify(&mut self, cmd: Cmd) {
		let WindowSize { width, height, .. } = Term::size();
		let area = crate::notify::Layout::available(Rect { x: 0, y: 0, width, height });

		self.cx.notify.tick(cmd, area);

		if self.cx.notify.messages.is_empty() {
			self.render();
		} else {
			self.render_notify();
		}
	}
}
