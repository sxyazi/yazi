use yazi_config::keymap::Exec;

use crate::input::{op::InputOp, Input, InputMode};

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Input {
	#[inline]
	pub fn visual(&mut self, _: impl Into<Opt>) -> bool {
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
