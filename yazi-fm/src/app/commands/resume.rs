use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;

use crate::{Term, app::App};

impl App {
	pub(crate) fn resume(&mut self, _: VoidOpt) -> Result<Data> {
		self.core.active_mut().preview.reset();
		self.term = Some(Term::start().unwrap());

		// While the app resumes, it's possible that the terminal size has changed.
		// We need to trigger a resize, and render the UI based on the resized area.
		act!(resize, self)?;

		self.signals.resume(None);

		succ!(render!());
	}
}
