use tokio::sync::mpsc;
use yazi_config::{YAZI, popup::InputCfg};
use yazi_macro::render;
use yazi_shared::{errors::InputError, event::CmdCow};
use yazi_widgets::input::InputCallback;

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
		let ticket = self.ticket.clone();

		// Reset input
		let cb: InputCallback = Box::new(move |before, after| {
			if opt.cfg.realtime {
				opt.tx.send(Err(InputError::Typed(format!("{before}{after}")))).ok();
			} else if opt.cfg.completion {
				opt.tx.send(Err(InputError::Completed(before.to_owned(), ticket.current()))).ok();
			}
		});
		self.inner = yazi_widgets::input::Input::new(
			opt.cfg.value,
			opt.cfg.position.offset.width.saturating_sub(YAZI.input.border()) as usize,
			opt.cfg.obscure,
			cb,
		);

		// Set cursor after reset
		// TODO: remove this
		if let Some(cursor) = opt.cfg.cursor {
			self.snap_mut().cursor = cursor;
			self.r#move(0);
		}

		render!();
	}
}
