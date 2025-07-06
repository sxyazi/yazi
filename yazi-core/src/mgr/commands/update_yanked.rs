use yazi_macro::render;
use yazi_parser::mgr::UpdateYankedOpt;

use crate::mgr::{Mgr, Yanked};

impl Mgr {
	pub fn update_yanked(&mut self, opt: impl TryInto<UpdateYankedOpt>) {
		let Ok(opt) = opt.try_into() else { return };

		if opt.urls.is_empty() && self.yanked.is_empty() {
			return;
		}

		self.yanked = Yanked::new(opt.cut, opt.urls);
		render!();
	}
}
