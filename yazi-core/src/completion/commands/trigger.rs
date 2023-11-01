use tracing::info;
use yazi_config::keymap::Exec;

use crate::completion::Completion;

pub struct Opt<'a> {
	word:   &'a str,
	ticket: usize,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self {
		Self {
			word:   e.args.get(0).map(|w| w.as_str()).unwrap_or_default(),
			ticket: e.named.get("ticket").and_then(|v| v.parse().ok()).unwrap_or(0),
		}
	}
}

impl Completion {
	pub fn trigger<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into();
		if self.ticket >= opt.ticket {
			return false;
		}

		self.close(false);
		self.ticket = opt.ticket;

		info!("trigger completion: {}", opt.word);
		tokio::spawn(async move {});
		false
	}
}
