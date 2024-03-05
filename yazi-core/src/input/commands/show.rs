use yazi_proxy::options::InputOpt;
use yazi_shared::render;

use crate::input::Input;

impl Input {
	pub fn show(&mut self, opt: impl TryInto<InputOpt>) {
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
