use yazi_config::keymap::Exec;

use crate::completion::Completion;

pub struct Opt<'a> {
	cands:   &'a Vec<String>,
	version: usize,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self {
		Self {
			cands:   &e.args,
			version: e.named.get("version").and_then(|v| v.parse().ok()).unwrap_or(0),
		}
	}
}

impl Completion {
	pub fn show<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into();
		if self.version != opt.version {
			return false;
		}

		self.close(false);
		self.items = opt.cands.clone();
		self.version = opt.version;
		self.visible = true;
		true
	}
}
