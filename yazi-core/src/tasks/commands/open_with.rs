use yazi_proxy::options::OpenWithOpt;

use crate::tasks::Tasks;

impl Tasks {
	pub fn open_with(&mut self, opt: impl TryInto<OpenWithOpt>) {
		if let Ok(opt) = opt.try_into() {
			self.process_from_opener(
				opt.cwd,
				opt.opener,
				opt.targets.into_iter().map(|u| u.into_path().into_os_string()).collect(),
			);
		}
	}
}
