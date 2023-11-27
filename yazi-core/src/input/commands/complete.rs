use std::path::MAIN_SEPARATOR;

use yazi_config::keymap::Exec;
use yazi_shared::Layer;

use crate::{emit, input::Input};

pub struct Opt<'a> {
	word:   &'a str,
	ticket: usize,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self {
		Self {
			word:   e.args.first().map(|w| w.as_str()).unwrap_or_default(),
			ticket: e.named.get("ticket").and_then(|s| s.parse().ok()).unwrap_or(0),
		}
	}
}

impl Input {
	#[inline]
	pub fn _complete(word: &str, ticket: usize) {
		emit!(Call(
			Exec::call("complete", vec![word.to_owned()]).with("ticket", ticket).vec(),
			Layer::Input
		));
	}

	pub fn complete<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into() as Opt;
		if self.ticket != opt.ticket {
			return false;
		}

		let [before, after] = self.partition();
		let new = if let Some((prefix, _)) = before.rsplit_once(MAIN_SEPARATOR) {
			format!("{prefix}/{}{after}", opt.word)
		} else {
			format!("{}{after}", opt.word)
		};

		let snap = self.snaps.current_mut();
		if new == snap.value {
			return false;
		}

		let delta = new.chars().count() as isize - snap.value.chars().count() as isize;
		snap.value = new;

		self.move_(delta);
		self.flush_value();
		true
	}
}
