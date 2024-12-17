use yazi_proxy::options::ProcessExecOpt;

use crate::tasks::Tasks;

impl Tasks {
	pub fn process_exec(&mut self, opt: impl TryInto<ProcessExecOpt>) {
		if let Ok(opt) = opt.try_into() {
			self.scheduler.process_open(opt);
		}
	}
}
