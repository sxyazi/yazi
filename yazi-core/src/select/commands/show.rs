use tokio::sync::oneshot;
use yazi_config::popup::SelectCfg;
use yazi_shared::{event::Cmd, render};

use crate::select::Select;

pub struct Opt {
	cfg: SelectCfg,
	tx:  oneshot::Sender<anyhow::Result<usize>>,
}

impl TryFrom<Cmd> for Opt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		Ok(Self { cfg: c.take_any("cfg").ok_or(())?, tx: c.take_any("tx").ok_or(())? })
	}
}

impl Select {
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
