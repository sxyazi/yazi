use yazi_shared::event::CmdCow;

use crate::app::App;

struct Opt;

impl From<CmdCow> for Opt {
	fn from(_: CmdCow) -> Self { Self }
}

impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl App {
	#[yazi_codegen::command]
	pub fn resize(&mut self, _: Opt) {
		self.reflow(());

		self.core.current_mut().sync_page(true);
		self.core.current_mut().arrow(0);
		self.core.mgr.peek(false);
		self.core.mgr.parent_mut().map(|f| f.arrow(0));
	}
}
