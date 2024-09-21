use yazi_shared::{event::Cmd, render};

use crate::input::{Input, InputMode, op::InputOp};

impl Input {
	#[inline]
	pub fn visual(&mut self, _: Cmd) {
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
