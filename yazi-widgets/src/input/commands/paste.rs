use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::input::PasteOpt;
use yazi_shared::event::Data;

use crate::{CLIPBOARD, input::{Input, op::InputOp}};

impl Input {
	pub fn paste(&mut self, opt: PasteOpt) -> Result<Data> {
		if let Some(start) = self.snap().op.start() {
			self.snap_mut().op = InputOp::Delete(false, false, start);
			self.handle_op(self.snap().cursor, true);
		}

		let s = futures::executor::block_on(CLIPBOARD.get());
		if s.is_empty() {
			succ!();
		}

		act!(insert, self, !opt.before)?;
		self.type_str(&s.to_string_lossy())?;
		act!(escape, self)?;
		succ!(render!());
	}
}
