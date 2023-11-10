use yazi_config::keymap::Exec;

use crate::input::Input;

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Input {
	#[inline]
	pub fn visual(&mut self, _: impl Into<Opt>) -> bool { self.snap_mut().visual() }
}
