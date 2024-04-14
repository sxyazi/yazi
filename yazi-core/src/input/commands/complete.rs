use std::path::MAIN_SEPARATOR_STR;

use yazi_shared::{event::Cmd, render};

use crate::input::Input;

pub struct Opt {
	word:   String,
	ticket: usize,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			word:   c.take_first().unwrap_or_default(),
			ticket: c.take_name("ticket").and_then(|s| s.parse().ok()).unwrap_or(0),
		}
	}
}

impl Input {
	pub fn complete(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		if self.ticket != opt.ticket {
			return;
		}

		let [before, after] = self.partition();
		let new = if let Some((prefix, _)) = before.rsplit_once(['/', '\\']) {
			format!("{prefix}/{}{after}", opt.word)
		} else {
			format!("{}{after}", opt.word)
		};

		let normalized_new = new.replace(['/', '\\'], MAIN_SEPARATOR_STR);

		let snap = self.snaps.current_mut();
		if normalized_new == snap.value {
			return;
		}

		let delta = normalized_new.chars().count() as isize - snap.value.chars().count() as isize;
		snap.value = normalized_new;

		self.move_(delta);
		self.flush_value();
		render!();
	}
}
