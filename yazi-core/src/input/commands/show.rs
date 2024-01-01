use anyhow::anyhow;
use tokio::sync::mpsc;
use yazi_config::popup::InputCfg;
use yazi_shared::{emit, event::Exec, render, InputError, Layer};

use crate::input::Input;

pub struct Opt {
	cfg: InputCfg,
	tx:  mpsc::UnboundedSender<Result<String, InputError>>,
}

impl TryFrom<&Exec> for Opt {
	type Error = anyhow::Error;

	fn try_from(e: &Exec) -> Result<Self, Self::Error> {
		e.take_data().ok_or_else(|| anyhow!("invalid data"))
	}
}

impl Input {
	pub fn _show(cfg: InputCfg) -> mpsc::UnboundedReceiver<Result<String, InputError>> {
		let (tx, rx) = mpsc::unbounded_channel();
		emit!(Call(Exec::call("show", vec![]).with_data(Opt { cfg, tx }).vec(), Layer::Input));
		rx
	}

	pub fn show(&mut self, opt: impl TryInto<Opt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

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
		render!();
	}
}
