use tokio::sync::oneshot;
use yazi_shared::{event::Exec, term::Term};

use crate::app::App;

pub struct Opt {
	state: bool,
	tx:    Option<oneshot::Sender<()>>,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self { state: e.args.first().map_or(false, |s| s == "true"), tx: e.take_data() }
	}
}

impl App {
	pub(crate) fn stop(&mut self, opt: impl TryInto<Opt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		self.cx.manager.active_mut().preview.reset_image();
		if opt.state {
			self.signals.stop_term(true);
			self.term = None;
		} else {
			self.term = Some(Term::start().unwrap());
			self.signals.stop_term(false);

			// While the app resumes, it's possible that the terminal size has changed.
			// We need to trigger a resize, and render the UI based on the resized area.
			self.resize().unwrap();
		}
		if let Some(tx) = opt.tx {
			tx.send(()).ok();
		}
	}
}
