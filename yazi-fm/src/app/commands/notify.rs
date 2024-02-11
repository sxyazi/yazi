use yazi_core::notify::Message;

use crate::app::App;

impl App {
	pub(crate) fn notify(&mut self, msg: impl TryInto<Message>) { self.cx.notify.push(msg); }
}
