use std::path::MAIN_SEPARATOR;

use yazi_shared::{emit, event::Exec, render, Layer};

use crate::input::Input;

pub struct Opt {
	word:   String,
	ticket: usize,
}

impl From<Exec> for Opt {
	fn from(mut e: Exec) -> Self {
		Self {
			word:   e.take_first().unwrap_or_default(),
			ticket: e.take_name("ticket").and_then(|s| s.parse().ok()).unwrap_or(0),
		}
	}
}

impl Input {
	#[inline]
	pub fn _complete(word: &str, ticket: usize) {
		emit!(Call(Exec::call("complete", vec![word.to_owned()]).with("ticket", ticket), Layer::Input));
	}

	pub fn complete(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		if self.ticket != opt.ticket {
			return;
		}

		let [before, after] = self.partition();
		let new = if let Some((prefix, _)) = before.rsplit_once(MAIN_SEPARATOR) {
			format!("{prefix}/{}{after}", opt.word)
		} else {
			format!("{}{after}", opt.word)
		};

		let snap = self.snaps.current_mut();
		if new == snap.value {
			return;
		}

		let delta = new.chars().count() as isize - snap.value.chars().count() as isize;
		snap.value = new;

		self.move_(delta);
		self.flush_value();
		render!();
	}
}
