use tokio::sync::oneshot;
use yazi_shared::event::Exec;

use crate::app::App;

pub struct Opt {
	tx: Option<oneshot::Sender<()>>,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { tx: e.take_data() } }
}

impl App {
	pub(crate) fn stop(&mut self, opt: impl Into<Opt>) {
		self.cx.manager.active_mut().preview.reset_image();

		self.signals.stop();
		self.term = None;

		if let Some(tx) = opt.into().tx {
			tx.send(()).ok();
		}
	}
}
