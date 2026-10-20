use serde::Serialize;

use crate::{Progress, TaskSummary};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct CustomProg {
	pub(crate) state:     Option<bool>,
	pub(crate) total:     u64,
	pub(crate) succeeded: u64,
	pub(crate) failed:    u64,
	pub(crate) workload:  u64,
	pub(crate) processed: u64,
	pub(crate) progress:  bool,
}

impl From<CustomProg> for TaskSummary {
	fn from(value: CustomProg) -> Self {
		Self {
			total_tasks:     1,
			succeeded_tasks: value.success() as u32,
			failed_tasks:    value.failed() as u32,
			total_items:     value.total,
			succeeded_items: value.succeeded,
			failed_items:    value.failed,
			percent:         value.percent().map(Into::into),
		}
	}
}

impl Progress for CustomProg {
	fn running(self) -> bool { self.state.is_none() }

	fn cooked(self) -> bool { self.state == Some(true) }

	fn failed(self) -> bool { self.state == Some(false) }

	fn percent(self) -> Option<f32> {
		self.progress.then(|| self.work_percent(self.processed, self.workload, self.total))
	}
}
