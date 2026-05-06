use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_shared::data::Data;

use crate::input::{INPUT_HISTORY, Input, InputOp, parser::HistoryOpt};

impl Input {
	pub fn history(&mut self, opt: HistoryOpt) -> Result<Data> {
		if self.snap().op != InputOp::None || self.obscure {
			succ!();
		}

		if !INPUT_HISTORY.lock().unwrap().navigate(opt.offset, &mut self.snaps, self.limit) {
			succ!();
		}

		succ!(render!());
	}
}
