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

	pub fn replace_str(&mut self, s: &str) {
		let snap = self.snap_mut();
		snap.mode = InputMode::Normal;

		let start = snap.idx(snap.cursor).unwrap();
		let mut it = snap.value[start..].char_indices();
		match (it.next(), it.next()) {
			(None, _) => {}
			(Some(_), None) => snap.value.replace_range(start..snap.len(), s),
			(Some(_), Some((len, _))) => snap.value.replace_range(start..start + len, s),
		}

		render!();
		self.snaps.tag(self.limit).then(|| self.flush_value());
	}
}
