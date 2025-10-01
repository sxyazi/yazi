use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::input::InsertOpt;
use yazi_shared::data::Data;

use crate::input::{Input, InputMode, op::InputOp};

impl Input {
	pub fn insert(&mut self, opt: InsertOpt) -> Result<Data> {
		let snap = self.snap_mut();
		if snap.mode == InputMode::Normal {
			snap.op = InputOp::None;
			snap.mode = InputMode::Insert;
		} else {
			succ!();
		}

		if opt.append {
			act!(r#move, self, 1)?;
		}

		succ!(render!());
	}
}
