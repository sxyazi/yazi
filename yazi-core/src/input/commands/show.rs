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
		let Ok(opt) = opt.try_into() else { return };

		self.close(false);
		self.visible = true;
		self.title = opt.cfg.title;
		self.position = opt.cfg.position;

		// Typing
		self.callback = Some(opt.tx);
		self.realtime = opt.cfg.realtime;
		self.completion = opt.cfg.completion;

		// Shell
		self.highlight = opt.cfg.highlight;

		// Reset snaps
		self.snaps.reset(opt.cfg.value, self.limit());

		// Set cursor after reset
		if let Some(cursor) = opt.cfg.cursor {
			self.snaps.current_mut().cursor = cursor;
			self.move_(0);
		}

		render!();
	}
}
