use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_shared::data::Data;

use crate::input::{Input, InputMode, InputOp};

impl Input {
	pub fn undo(&mut self, _: ()) -> Result<Data> {
		if self.snap().op != InputOp::None {
			succ!();
		}

		if !self.snaps.undo() {
			succ!();
		}

		act!(r#move, self)?;
		if self.snap().mode == InputMode::Insert {
			act!(escape, self)?;
		}

		succ!(render!());
	}
}
