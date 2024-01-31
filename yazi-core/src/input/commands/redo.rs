use yazi_shared::{event::Cmd, render};

use crate::input::Input;

impl Input {
	pub fn redo(&mut self, _: Cmd) {
		render!(self.snaps.redo());
	}
}
