use yazi_shared::event::Exec;

use crate::input::Input;

impl Input {
	pub fn redo(&mut self, _: &Exec) -> bool { self.snaps.redo() }
}
