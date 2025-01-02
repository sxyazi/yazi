use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::input::{Input, InputMode, op::InputOp};

impl Input {
	#[yazi_codegen::command]
	pub fn replace(&mut self, _: CmdCow) {
		let snap = self.snap_mut();
		if snap.mode == InputMode::Normal {
			snap.op = InputOp::None;
			snap.mode = InputMode::Replace;
		} else {
			return;
		}

		render!();
	}
}

