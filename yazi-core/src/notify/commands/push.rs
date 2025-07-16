use std::time::{Duration, Instant};

use yazi_parser::app::NotifyOpt;
use yazi_proxy::AppProxy;

use crate::notify::{Message, Notify};

impl Notify {
	pub fn push(&mut self, opt: NotifyOpt) {
		let instant = Instant::now();

		let mut msg = Message::from(opt);
		msg.timeout += instant - self.messages.first().map_or(instant, |m| m.instant);

		if self.messages.iter().all(|m| m != &msg) {
			self.messages.push(msg);
			AppProxy::update_notify(Duration::ZERO);
		}
	}
}
