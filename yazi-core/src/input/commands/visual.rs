use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::input::{Input, InputMode, op::InputOp};

impl Input {
	pub fn visual(&mut self, _: CmdCow) {
		if self.snap().mode != InputMode::Normal {
			self.escape(());
		}

		let snap = self.snap_mut();
		if !snap.value.is_empty() {
			snap.op = InputOp::Select(snap.cursor);
			render!();
		}
	}
}
