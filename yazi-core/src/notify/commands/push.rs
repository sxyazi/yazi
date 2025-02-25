use std::time::Instant;

use yazi_macro::emit;
use yazi_shared::event::Cmd;

use crate::notify::{Message, Notify};

impl Notify {
	pub fn push(&mut self, msg: impl Into<Message>) {
		let mut msg = msg.into() as Message;

		let instant = Instant::now();
		msg.timeout += instant - self.messages.first().map_or(instant, |m| m.instant);

		if self.messages.iter().all(|m| m != &msg) {
			self.messages.push(msg);
			emit!(Call(Cmd::args("app:update_notify", &[0])));
		}
	}
}
