use tokio::sync::oneshot;
use yazi_shared::event::Cmd;

use crate::app::App;

pub struct Opt {
	tx: Option<oneshot::Sender<()>>,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self { Self { tx: c.take_any("tx") } }
}

impl App {
	pub(crate) fn stop(&mut self, opt: impl Into<Opt>) {
		self.cx.manager.active_mut().preview.reset_image();

		self.signals.stop(opt.into().tx);
		self.term = None;
	}
}
