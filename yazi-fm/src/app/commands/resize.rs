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
		self.cx.active_mut().preview.reset();
		self.reflow(());

		self.cx.current_mut().sync_page(true);
		self.cx.current_mut().arrow(0);
		self.cx.mgr.peek(false);
		self.cx.mgr.parent_mut().map(|f| f.arrow(0));
	}
}
