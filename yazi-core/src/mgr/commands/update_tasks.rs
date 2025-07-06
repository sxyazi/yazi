use yazi_parser::mgr::UpdateTasksOpt;

use crate::mgr::Mgr;

impl Mgr {
	pub fn update_tasks(&mut self, opt: impl TryInto<UpdateTasksOpt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		self.watcher.push_files(opt.urls);
	}
}
