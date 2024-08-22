use tokio::sync::oneshot;
use yazi_config::popup::ConfirmCfg;
use yazi_shared::{event::Cmd, render};

use crate::confirm::Confirm;

pub struct Opt {
	cfg: ConfirmCfg,
	tx:  oneshot::Sender<bool>,
}

impl TryFrom<Cmd> for Opt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		Ok(Self { cfg: c.take_any("cfg").ok_or(())?, tx: c.take_any("tx").ok_or(())? })
	}
}

impl Confirm {
	pub fn show(&mut self, opt: impl TryInto<Opt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		self.close(false);
		self.title = opt.cfg.title;
		self.content = opt.cfg.content;
		self.list = opt.cfg.list;

		self.offset = 0;
		self.position = opt.cfg.position;

		self.callback = Some(opt.tx);
		self.visible = true;
		render!();
	}
}
