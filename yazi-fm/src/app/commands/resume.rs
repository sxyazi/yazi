use yazi_shared::{event::Exec, term::Term};

use crate::app::App;

impl App {
	pub(crate) fn resume(&mut self, _: Exec) {
		self.cx.manager.active_mut().preview.reset_image();
		self.term = Some(Term::start().unwrap());

		// While the app resumes, it's possible that the terminal size has changed.
		// We need to trigger a resize, and render the UI based on the resized area.
		self.resize().unwrap();

		self.signals.resume();
	}
}
