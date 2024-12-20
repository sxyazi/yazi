use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::{Term, app::App};

impl App {
	pub(crate) fn resume(&mut self, _: CmdCow) {
		self.cx.active_mut().preview.reset_image();
		self.term = Some(Term::start().unwrap());

		// While the app resumes, it's possible that the terminal size has changed.
		// We need to trigger a resize, and render the UI based on the resized area.
		self.resize(());

		self.signals.resume(None);

		render!();
	}
}
