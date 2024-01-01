use yazi_shared::{event::Exec, render};

use crate::input::Input;

impl Input {
	pub fn redo(&mut self, _: &Exec) {
		render!(self.snaps.redo());
	}
}
