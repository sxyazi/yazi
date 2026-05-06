use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_shared::data::Data;

use crate::input::{INPUT_HISTORY, Input, InputOp, parser::HistoryOpt};

impl Input {
	pub fn history(&mut self, opt: HistoryOpt) -> Result<Data> {
		if self.snap().op != InputOp::None || self.obscure {
			succ!();
		}

		let new_value = INPUT_HISTORY.lock().unwrap().navigate(opt.offset, &self.snap().value);
		let Some(value) = new_value else { succ!() };

		let snap = self.snap_mut();
		snap.value = value;
		snap.cursor = snap.count();

		act!(r#move, self)?;
		succ!(render!());
	}
}
