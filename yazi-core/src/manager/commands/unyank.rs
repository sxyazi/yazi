use yazi_shared::{event::Cmd, render};

use crate::manager::Manager;

pub struct Opt;

impl From<Cmd> for Opt {
	fn from(_: Cmd) -> Self { Self }
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Manager {
	pub fn unyank(&mut self, _: impl Into<Opt>) {
		render!(!self.yanked.is_empty());

		self.yanked = Default::default();
	}
}
