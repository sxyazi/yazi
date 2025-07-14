use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidOpt;
use yazi_shared::event::Data;

use crate::input::{Input, InputMode};

impl Input {
	pub fn undo(&mut self, _: VoidOpt) -> Result<Data> {
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
