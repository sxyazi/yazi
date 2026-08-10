use yazi_macro::impl_data_any;
use yazi_shared::url::UrlBuf;

use crate::{Task, TaskProg, TasksProxy};

#[derive(Clone, Debug)]
pub enum CustomOut {
	Progress { total: u32, success: u32, failed: u32, workload: u64, processed: u64 },
	Log(String),
	Succ(Vec<UrlBuf>),
	Fail(String),
}

impl_data_any!(CustomOut);

impl CustomOut {
	pub(crate) fn reduce(self, task: &mut Task) {
		match self {
			Self::Progress { total, success, failed, workload, processed } => {
				let TaskProg::Custom(prog) = &mut task.prog else { return };
				prog.total = prog.total.saturating_add(total);
				prog.success = prog.success.saturating_add(success);
				prog.failed = prog.failed.saturating_add(failed);
				prog.total = prog.success.saturating_add(prog.failed).max(prog.total);

				prog.workload = prog.workload.saturating_add(workload);
				prog.processed = prog.processed.saturating_add(processed);
				prog.workload = prog.processed.max(prog.workload);
			}
			Self::Log(line) => task.log(line),
			Self::Succ(urls) => {
				let TaskProg::Custom(prog) = &mut task.prog else { return };
				let None = prog.state else { return };
				prog.state = Some(true);
				TasksProxy::update_succeed(task.id, urls, true);
			}
			Self::Fail(reason) => {
				let TaskProg::Custom(prog) = &mut task.prog else { return };
				let None = prog.state else { return };
				prog.state = Some(false);
				task.log(reason);
			}
		}
	}
}
