use std::path::MAIN_SEPARATOR_STR;

use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::input::CompleteOpt;
use yazi_shared::data::Data;

use crate::input::Input;

#[cfg(windows)]
const SEPARATOR: [char; 2] = ['/', '\\'];

#[cfg(not(windows))]
const SEPARATOR: char = std::path::MAIN_SEPARATOR;

impl Input {
	pub fn complete(&mut self, opt: CompleteOpt) -> Result<Data> {
		let (before, after) = self.partition();
		let new = if let Some((prefix, _)) = before.rsplit_once(SEPARATOR) {
			format!("{prefix}/{}{after}", opt.item.completable()).replace(SEPARATOR, MAIN_SEPARATOR_STR)
		} else {
			format!("{}{after}", opt.item.completable()).replace(SEPARATOR, MAIN_SEPARATOR_STR)
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
