use yazi_shared::event::Cmd;

use crate::app::App;

pub struct Opt;

impl From<Cmd> for Opt {
	fn from(_: Cmd) -> Self { Self }
}

impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl App {
	#[yazi_codegen::command]
	pub fn resize(&mut self, _: Opt) {
		self.cx.manager.active_mut().preview.reset();
		self.render();

		self.cx.manager.current_mut().sync_page(true);
		self.cx.manager.hover(None);
	}
}
