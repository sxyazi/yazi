use yazi_shared::event::Exec;

use crate::app::App;

pub struct Opt;

impl From<Exec> for Opt {
	fn from(_: Exec) -> Self { Self }
}

impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl App {
	pub(crate) fn resize(&mut self, _: impl Into<Opt>) {
		self.cx.manager.active_mut().preview.reset();
		self.render();

		self.cx.manager.current_mut().sync_page(true);
		self.cx.manager.hover(None);
	}
}
