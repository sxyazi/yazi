use yazi_macro::render;
use yazi_shared::event::Cmd;

use crate::input::{Input, InputMode};

impl Input {
	pub fn undo(&mut self, _: Cmd) {
		if !self.snaps.undo() {
			return;
		}
		if self.snap().mode == InputMode::Insert {
			self.escape(());
		}
		render!();
	}
}
