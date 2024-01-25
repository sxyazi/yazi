use tokio::sync::mpsc;
use yazi_config::popup::InputCfg;
use yazi_shared::{emit, event::Exec, render, InputError, Layer};

use crate::input::Input;

pub struct Opt {
	cfg: InputCfg,
	tx:  mpsc::UnboundedSender<Result<String, InputError>>,
}

impl TryFrom<Exec> for Opt {
	type Error = ();

	fn try_from(mut e: Exec) -> Result<Self, Self::Error> { e.take_data().ok_or(()) }
}

impl Input {
	pub fn _show(cfg: InputCfg) -> mpsc::UnboundedReceiver<Result<String, InputError>> {
		let (tx, rx) = mpsc::unbounded_channel();
		emit!(Call(Exec::call("show", vec![]).with_data(Opt { cfg, tx }), Layer::Input));
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

		// Set cursor after reset
		if let Some(cursor) = opt.cfg.cursor {
			self.snaps.current_mut().cursor = cursor;
			self.move_(0);
		}

		render!();
	}
}
