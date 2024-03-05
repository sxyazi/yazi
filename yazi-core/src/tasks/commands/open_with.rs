use yazi_proxy::options::OpenWithOpt;

use crate::tasks::Tasks;

impl Tasks {
	pub fn open_with(&mut self, opt: impl TryInto<OpenWithOpt>) {
		if let Ok(opt) = opt.try_into() {
			self.file_open_with(&opt.opener, &opt.targets);
		}
	}
}
