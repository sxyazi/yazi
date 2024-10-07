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
	#[yazi_macro::command]
	pub fn toggle(&mut self, _: Opt) {
		self.visible = !self.visible;

		if self.visible {
			self.summaries = self.paginate();
			self.arrow(0);
		}

		render!();
	}
}
