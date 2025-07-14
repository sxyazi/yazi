use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidOpt;
use yazi_shared::event::Data;

use crate::input::{Input, InputMode, op::InputOp};

impl Input {
	pub fn visual(&mut self, _: VoidOpt) -> Result<Data> {
		if self.snap().mode != InputMode::Normal {
			act!(escape, self)?;
		}

		let snap = self.snap_mut();
		if !snap.value.is_empty() {
			snap.op = InputOp::Select(snap.cursor);
			render!();
		}
		succ!();
	}
}
