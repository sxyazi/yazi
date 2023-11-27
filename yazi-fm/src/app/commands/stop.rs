use anyhow::Result;
use tokio::sync::oneshot;
use yazi_core::{emit, manager::Manager};
use yazi_shared::{term::Term, Exec};

use crate::app::App;

pub struct Opt {
	state: bool,
	tx:    Option<oneshot::Sender<()>>,
}

impl TryFrom<&Exec> for Opt {
	type Error = anyhow::Error;

	fn try_from(e: &Exec) -> Result<Self, Self::Error> {
		Ok(Self { state: e.args.first().map_or(false, |s| s == "true"), tx: e.take_data() })
	}
}

impl App {
	pub(crate) fn stop(&mut self, opt: impl TryInto<Opt>) -> bool {
		let Ok(opt) = opt.try_into() else {
			return false;
		};

		self.cx.manager.active_mut().preview.reset_image();
		if opt.state {
			self.signals.stop_term(true);
			self.term = None;
		} else {
			self.term = Some(Term::start().unwrap());
			self.signals.stop_term(false);
			emit!(Render);
			Manager::_hover(None);
		}
		if let Some(tx) = opt.tx {
			tx.send(()).ok();
		}
		false
	}
}
