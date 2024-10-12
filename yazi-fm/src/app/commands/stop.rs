use tokio::sync::oneshot;
use yazi_shared::event::Cmd;

use crate::app::App;

struct Opt {
	tx: Option<oneshot::Sender<()>>,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self { Self { tx: c.take_any("tx") } }
}

impl App {
	#[yazi_codegen::command]
	pub fn stop(&mut self, opt: Opt) {
		self.cx.manager.active_mut().preview.reset_image();

		// We need to destroy the `term` first before stopping the `signals`
		// to prevent any signal from triggering the term to render again
		// while the app is being suspended.
		self.term = None;

		self.signals.stop(opt.tx);
	}
}
