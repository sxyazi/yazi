use yazi_macro::render;
use yazi_shared::event::Cmd;

use crate::input::Input;

impl Input {
	pub fn redo(&mut self, _: Cmd) {
		render!(self.snaps.redo());
	}
}
