use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::input::BackspaceOpt;
use yazi_shared::data::Data;

use crate::input::Input;

impl Input {
	pub fn backspace(&mut self, opt: BackspaceOpt) -> Result<Data> {
		let snap = self.snap_mut();
		if !opt.under && snap.cursor < 1 {
			succ!();
		} else if opt.under && snap.cursor >= snap.count() {
			succ!();
		}

		if opt.under {
			snap.value.remove(snap.idx(snap.cursor).unwrap());
			act!(r#move, self)?;
		} else {
			snap.value.remove(snap.idx(snap.cursor - 1).unwrap());
			act!(r#move, self, -1)?;
		}

		self.flush_value();
		succ!(render!());
	}
}
