use yazi_macro::render;
use yazi_parser::pick::ShowOpt;

use crate::pick::Pick;

impl Pick {
	pub fn show(&mut self, opt: impl TryInto<ShowOpt>) {
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
