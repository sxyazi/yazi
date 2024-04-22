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
		self.yanked.clear();
		render!(self.yanked.catchup_revision(false));
	}
}
