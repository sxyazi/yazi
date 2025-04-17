use ratatui::layout::Rect;
use yazi_adapter::Dimension;
use yazi_shared::event::CmdCow;

use crate::{app::App, notify};

impl App {
	pub(crate) fn update_notify(&mut self, cmd: CmdCow) {
		let Dimension { rows, columns, .. } = Dimension::available();
		let area =
			notify::Notify::available(Rect { x: 0, y: 0, width: columns, height: rows });

		self.cx.notify.tick(cmd, area);

		if self.cx.notify.messages.is_empty() {
			self.render();
		} else {
			self.render_partially();
		}
	}
}
