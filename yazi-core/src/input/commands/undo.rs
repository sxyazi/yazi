use yazi_shared::{event::Exec, render};

use crate::input::{Input, InputMode};

impl Input {
	pub fn undo(&mut self, _: &Exec) {
		if !self.snaps.undo() {
			return;
		}
		if self.snap().mode == InputMode::Insert {
			self.escape(());
		}
		render!();
	}
}
