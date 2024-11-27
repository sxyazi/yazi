use tokio::sync::oneshot;
use yazi_shared::event::CmdCow;

use crate::app::App;

struct Opt {
	tx: Option<oneshot::Sender<()>>,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self { Self { tx: c.take_any("tx") } }
}

impl App {
	#[yazi_codegen::command]
	pub fn stop(&mut self, opt: Opt) {
		self.cx.active_mut().preview.reset_image();

		// We need to destroy the `term` first before stopping the `signals`
		// to prevent any signal from triggering the term to render again
		// while the app is being suspended.
		self.term = None;

		self.signals.stop(opt.tx);
	}
}
