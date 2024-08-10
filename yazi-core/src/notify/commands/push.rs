use std::time::Instant;

use yazi_shared::{emit, event::Cmd, Layer};

use crate::notify::{Message, Notify};

impl Notify {
	pub fn push(&mut self, msg: impl Into<Message>) {
		let mut msg = msg.into() as Message;

		let instant = Instant::now();
		msg.timeout += instant - self.messages.first().map_or(instant, |m| m.instant);
		self.messages.push(msg);

		emit!(Call(Cmd::args("update_notify", &[0]), Layer::App));
	}
}
