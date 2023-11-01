use yazi_config::keymap::Exec;

use crate::completion::Completion;

pub struct Opt<'a> {
	cands:  &'a Vec<String>,
	ticket: usize,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self {
		Self {
			cands:  &e.args,
			ticket: e.named.get("ticket").and_then(|v| v.parse().ok()).unwrap_or(0),
		}
	}
}

impl Completion {
	pub fn show<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into();
		if self.ticket != opt.ticket {
			return false;
		}

		self.close(false);
		self.items = opt.cands.clone();
		self.ticket = opt.ticket;
		self.visible = true;
		true
	}
}
