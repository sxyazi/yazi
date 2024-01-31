use yazi_shared::{event::Cmd, render};

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
