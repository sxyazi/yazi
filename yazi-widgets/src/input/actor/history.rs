use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_shared::data::Data;

use crate::input::{Input, InputHistory, InputOp, parser::HistoryOpt};

impl Input {
	pub fn history(&mut self, opt: HistoryOpt, history: &mut InputHistory) -> Result<Data> {
		if self.snap().op != InputOp::None || self.obscure {
			succ!();
		}

		if !history.navigate(opt.offset, &mut self.snaps, self.limit) {
			succ!();
		}

		succ!(render!());
	}
}
