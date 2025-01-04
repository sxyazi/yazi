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
			render!();
		}
	}

	// FIXME: need to create a new snap for each replace operation, otherwise we
	// can't undo/redo it with `u` and `Ctrl-r`
	pub fn replace_str(&mut self, s: &str) {
		let snap = self.snap_mut();

		let start = snap.idx(snap.cursor).unwrap();
		let end = snap.idx(snap.cursor + 1).unwrap(); // FIXME: panic if the input is empty while in replace mode

		snap.mode = InputMode::Normal;
		snap.value.replace_range(start..end, s);

		self.flush_value();
		render!();
	}
}
