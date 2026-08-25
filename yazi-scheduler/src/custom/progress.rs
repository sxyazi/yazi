use serde::Serialize;

use crate::{Progress, TaskSummary};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct CustomProg {
	pub(crate) state:     Option<bool>,
	pub(crate) total:     u32,
	pub(crate) success:   u32,
	pub(crate) failed:    u32,
	pub(crate) workload:  u64,
	pub(crate) processed: u64,
	pub(crate) progress:  bool,
}

impl From<CustomProg> for TaskSummary {
	fn from(value: CustomProg) -> Self {
		if value.progress {
			Self {
				total:   value.total,
				success: value.success,
				failed:  value.failed,
				percent: value.percent().map(Into::into),
			}
		} else {
			Self {
				total:   1,
				success: value.success() as u32,
				failed:  value.failed() as u32,
				percent: None,
			}
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
