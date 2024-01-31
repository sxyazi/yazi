use yazi_shared::{event::Cmd, render};

use crate::tasks::Tasks;

pub struct Opt;

impl From<Cmd> for Opt {
	fn from(_: Cmd) -> Self { Self }
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Tasks {
	pub fn toggle(&mut self, _: impl Into<Opt>) {
		self.visible = !self.visible;
		render!();
	}
}
