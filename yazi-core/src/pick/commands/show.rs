use tokio::sync::oneshot;
use yazi_config::popup::PickCfg;
use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::pick::Pick;

pub struct Opt {
	cfg: PickCfg,
	tx:  oneshot::Sender<anyhow::Result<usize>>,
}

impl TryFrom<CmdCow> for Opt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { cfg: c.take_any("cfg").ok_or(())?, tx: c.take_any("tx").ok_or(())? })
	}
}

impl Pick {
	pub fn show(&mut self, opt: impl TryInto<Opt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		self.close(false);
		self.title = opt.cfg.title;
		self.items = opt.cfg.items;
		self.position = opt.cfg.position;

		self.callback = Some(opt.tx);
		self.visible = true;
		render!();
	}
}
