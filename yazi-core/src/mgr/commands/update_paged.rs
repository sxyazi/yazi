use yazi_parser::mgr::UpdatePagedOpt;

use crate::{mgr::Mgr, tasks::Tasks};

impl Mgr {
	pub fn update_paged(&mut self, opt: impl TryInto<UpdatePagedOpt>, tasks: &Tasks) {
		let Ok(opt): Result<UpdatePagedOpt, _> = opt.try_into() else {
			return;
		};

		if opt.only_if.is_some_and(|u| u != *self.cwd()) {
			return;
		}

		let targets = self.current().paginate(opt.page.unwrap_or(self.current().page));
		if !targets.is_empty() {
			tasks.fetch_paged(targets, &self.mimetype);
			tasks.preload_paged(targets, &self.mimetype);
		}
	}
}
