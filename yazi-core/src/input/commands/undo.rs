use yazi_shared::event::Exec;

use crate::input::{Input, InputMode};

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Input {
	pub fn undo(&mut self, _: impl Into<Opt>) -> bool {
		if !self.snaps.undo() {
			return false;
		}
		if self.snap().mode == InputMode::Insert {
			self.escape(());
		}
		true
	}
}
