use std::path::MAIN_SEPARATOR_STR;

use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_shared::data::Data;

use crate::input::{Input, SEPARATOR, parser::CompleteOpt};

impl Input {
	pub fn complete(&mut self, opt: CompleteOpt) -> Result<Data> {
		let (before, after) = self.partition();
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
		self.flush_type();
		succ!(render!());
	}
}
