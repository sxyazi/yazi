use yazi_proxy::SelectOpt;
use yazi_shared::render;

use crate::select::Select;

impl Select {
	pub fn show(&mut self, opt: impl TryInto<SelectOpt>) {
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
