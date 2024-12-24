use tokio::sync::oneshot;
use yazi_config::popup::ConfirmCfg;
use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::confirm::Confirm;

pub struct Opt {
	cfg: ConfirmCfg,
	tx:  oneshot::Sender<bool>,
}

impl TryFrom<CmdCow> for Opt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
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

		self.position = opt.cfg.position;
		self.offset = 0;

		self.callback = Some(opt.tx);
		self.visible = true;
		render!();
	}
}
