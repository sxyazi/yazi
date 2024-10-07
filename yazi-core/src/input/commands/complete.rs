use std::path::MAIN_SEPARATOR_STR;

use yazi_shared::{event::{Cmd, Data}, render};

use crate::input::Input;

#[cfg(windows)]
const SEPARATOR: [char; 2] = ['/', '\\'];

#[cfg(not(windows))]
const SEPARATOR: char = std::path::MAIN_SEPARATOR;

pub struct Opt {
	word:   String,
	ticket: usize,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			word:   c.take_first_str().unwrap_or_default(),
			ticket: c.get("ticket").and_then(Data::as_usize).unwrap_or(0),
		}
	}
}

impl Input {
	#[yazi_macro::command]
	pub fn complete(&mut self, opt: Opt) {
		if self.ticket != opt.ticket {
			return;
		}

		let [before, after] = self.partition();
		let new = if let Some((prefix, _)) = before.rsplit_once(SEPARATOR) {
			format!("{prefix}/{}{after}", opt.word).replace(SEPARATOR, MAIN_SEPARATOR_STR)
		} else {
			format!("{}{after}", opt.word).replace(SEPARATOR, MAIN_SEPARATOR_STR)
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
