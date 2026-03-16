use std::path::MAIN_SEPARATOR_STR;

use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_shared::data::Data;

use crate::input::{Input, parser::CompleteOpt};

#[cfg(windows)]
const SEPARATOR: [char; 2] = ['/', '\\'];

#[cfg(not(windows))]
const SEPARATOR: char = std::path::MAIN_SEPARATOR;

impl Input {
	pub fn complete(&mut self, opt: CompleteOpt) -> Result<Data> {
		let (before, after) = self.partition();

		// Strip the remainder of the current path component from `after`, so that
		// completing when the cursor is in the middle of a word replaces the entire
		// word instead of appending the completion before the leftover suffix.
		// e.g. input "/home/user/D|oc" (cursor at |) completing "Documents/" should
		// yield "/home/user/Documents/", not "/home/user/Documents/oc".
		let after = match after.find(SEPARATOR) {
			Some(i) => &after[i..],
			None => "",
		};

		let new = if let Some((prefix, _)) = before.rsplit_once(SEPARATOR) {
			format!("{prefix}/{}{after}", opt.completable()).replace(SEPARATOR, MAIN_SEPARATOR_STR)
		} else {
			format!("{}{after}", opt.completable()).replace(SEPARATOR, MAIN_SEPARATOR_STR)
		};

		let snap = self.snap_mut();
		if new == snap.value {
			succ!();
		}

		let delta = new.chars().count() as isize - snap.count() as isize;
		snap.value = new;

		act!(r#move, self, delta)?;
		self.flush_value();
		succ!(render!());
	}
}
