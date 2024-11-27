use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::input::Input;

impl Input {
	pub fn redo(&mut self, _: CmdCow) {
		render!(self.snaps.redo());
	}
}
