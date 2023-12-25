use yazi_shared::event::Exec;

use crate::input::{Input, InputMode};

impl Input {
	pub fn undo(&mut self, _: &Exec) -> bool {
		if !self.snaps.undo() {
			return false;
		}
		if self.snap().mode == InputMode::Insert {
			self.escape(());
		}
		true
	}
}
