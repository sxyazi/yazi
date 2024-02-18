use std::time::Instant;

use yazi_shared::{emit, event::Cmd, Layer};

use crate::notify::{Message, Notify};

impl Notify {
	#[inline]
	pub fn _push_warn(title: &str, content: &str) {
		emit!(Call(
			Cmd::new("notify")
				.with("title", title)
				.with("content", content)
				.with("level", "warn")
				.with("timeout", 5),
			Layer::App
		));
	}

	pub fn push(&mut self, msg: impl TryInto<Message>) {
		let Ok(mut msg) = msg.try_into() else {
			return;
		};

		let instant = Instant::now();
		msg.timeout += instant - self.messages.first().map_or(instant, |m| m.instant);
		self.messages.push(msg);

		emit!(Call(Cmd::args("update_notify", vec![0.to_string()]), Layer::App));
	}
}
