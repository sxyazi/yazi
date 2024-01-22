use yazi_shared::{event::Exec, render};

use crate::input::{op::InputOp, Input, InputMode};

impl Input {
	#[inline]
	pub fn visual(&mut self, _: Exec) {
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
