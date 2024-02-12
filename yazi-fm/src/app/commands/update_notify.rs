use yazi_shared::event::Cmd;

use crate::app::App;

impl App {
	pub(crate) fn update_notify(&mut self, cmd: Cmd) {
		self.cx.notify.tick(cmd);

		if self.cx.notify.messages.is_empty() {
			self.render();
		} else {
			self.render_notify();
		}
	}
}
