use tracing::info;
use yazi_config::keymap::Exec;

use crate::completion::Completion;

pub struct Opt<'a> {
	word:    &'a str,
	version: usize,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self {
		Self {
			word:    e.args.get(0).map(|w| w.as_str()).unwrap_or_default(),
			version: e.named.get("version").and_then(|v| v.parse().ok()).unwrap_or(0),
		}
	}
}

impl Completion {
	pub fn trigger<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into();
		if self.version >= opt.version {
			return false;
		}

		self.close(false);
		self.version = opt.version;

		info!("trigger completion: {}", opt.word);
		tokio::spawn(async move {});
		false
	}
}
