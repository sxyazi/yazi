use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::app::StopOpt;
use yazi_shared::data::Data;

use crate::app::App;

impl App {
	pub fn stop(&mut self, opt: StopOpt) -> Result<Data> {
		self.core.active_mut().preview.reset_image();

		// We need to destroy the `term` first before stopping the `signals`
		// to prevent any signal from triggering the term to render again
		// while the app is being suspended.
		self.term = None;

		self.signals.stop(opt.tx);

		succ!();
	}
}
