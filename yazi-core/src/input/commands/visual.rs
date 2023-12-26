use yazi_shared::event::Exec;

use crate::input::{op::InputOp, Input, InputMode};

impl Input {
	#[inline]
	pub fn visual(&mut self, _: &Exec) -> bool {
		let snap = self.snap_mut();
		if snap.mode != InputMode::Normal {
			return false;
		} else if snap.value.is_empty() {
			return false;
		}

		snap.op = InputOp::Select(snap.cursor);
		true
	}
}
