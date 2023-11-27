use yazi_shared::event::Exec;

use crate::input::Input;

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Input {
	pub fn redo(&mut self, _: impl Into<Opt>) -> bool { self.snaps.redo() }
}
