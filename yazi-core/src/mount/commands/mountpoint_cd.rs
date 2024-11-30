use yazi_proxy::options::ProcessExecOpt;

use crate::mount::Mount;

impl Mount {
	pub fn mountpoint_cd(&mut self, opt: impl TryInto<ProcessExecOpt>) {
		if let Ok(opt) = opt.try_into() {}
	}
}
