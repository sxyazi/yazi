use yazi_shared::{event::Exec, render};

use crate::tasks::Tasks;

pub struct Opt;

impl From<Exec> for Opt {
	fn from(_: Exec) -> Self { Self }
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
