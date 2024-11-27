use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::input::{Input, InputMode, op::InputOp};

impl Input {
	#[inline]
	pub fn visual(&mut self, _: CmdCow) {
		let snap = self.snap_mut();
		if snap.mode != InputMode::Normal {
			return;
		} else if snap.value.is_empty() {
			return;
		}

		snap.op = InputOp::Select(snap.cursor);
		render!();
	}
}
