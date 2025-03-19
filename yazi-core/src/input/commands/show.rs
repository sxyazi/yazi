use tokio::sync::mpsc;
use yazi_config::popup::InputCfg;
use yazi_macro::render;
use yazi_shared::{errors::InputError, event::CmdCow};

use crate::input::Input;

pub struct Opt {
	cfg: InputCfg,
	tx:  mpsc::UnboundedSender<Result<String, InputError>>,
}

impl TryFrom<CmdCow> for Opt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { cfg: c.take_any("cfg").ok_or(())?, tx: c.take_any("tx").ok_or(())? })
	}
}

impl Input {
	pub fn show(&mut self, opt: impl TryInto<Opt>) {
		let Ok(opt): Result<Opt, _> = opt.try_into() else { return };

		self.close(false);
		self.visible = true;
		self.title = opt.cfg.title;
		self.position = opt.cfg.position;

		// Typing
		self.tx = Some(opt.tx.clone());

		// Shell
		self.highlight = opt.cfg.highlight;

		// Reset input
		let ticket = self.ticket.clone();
		let cb: Box<dyn Fn(&str, &str)> = Box::new(move |before, after| {
			if opt.cfg.realtime {
				opt.tx.send(Err(InputError::Typed(format!("{before}{after}")))).ok();
			}

			if opt.cfg.completion {
				opt.tx.send(Err(InputError::Completed(before.to_owned(), ticket.current()))).ok();
			}
		});
		self.inner = yazi_widgets::input::Input::new(opt.cfg.value, self.limit, cb);

		// Set cursor after reset
		// TODO: remove this
		if let Some(cursor) = opt.cfg.cursor {
			self.snap_mut().cursor = cursor;
			self.move_(0);
		}

		render!();
	}
}
