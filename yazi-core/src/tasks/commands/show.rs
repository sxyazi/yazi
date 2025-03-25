use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::tasks::Tasks;

struct Opt;

impl From<CmdCow> for Opt {
	fn from(_: CmdCow) -> Self { Self }
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Tasks {
	#[yazi_codegen::command]
	pub fn show(&mut self, _: Opt) {
		if self.visible {
			return;
		}

		self.visible = true;
		self.summaries = self.paginate();

		self.arrow(0);
		render!();
	}
}
