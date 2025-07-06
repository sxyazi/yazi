use yazi_macro::render;
use yazi_parser::confirm::ShowOpt;

use crate::confirm::Confirm;

impl Confirm {
	pub fn show(&mut self, opt: impl TryInto<ShowOpt>) {
		let Ok(opt): Result<ShowOpt, _> = opt.try_into() else {
			return;
		};

		self.close(false);
		self.title = opt.cfg.title;
		self.body = opt.cfg.body;
		self.list = opt.cfg.list;

		self.position = opt.cfg.position;
		self.offset = 0;

		self.callback = Some(opt.tx);
		self.visible = true;
		render!();
	}
}
