use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::mount::Mount;

struct Opt;

impl From<CmdCow> for Opt {
	fn from(_: CmdCow) -> Self { Self }
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Mount {
	#[yazi_codegen::command]
	pub fn toggle(&mut self, _: Opt) {
		self.visible = !self.visible;

		if self.visible {
			self.arrow(0);
		}

		render!();
	}
}
