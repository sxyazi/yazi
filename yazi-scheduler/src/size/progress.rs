use serde::Serialize;

use crate::{Progress, TaskSummary};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct SizeProg {
	pub(crate) done: bool,
}

impl From<SizeProg> for TaskSummary {
	fn from(value: SizeProg) -> Self {
		Self {
			total_tasks:     1,
			succeeded_tasks: value.success() as u32,
			failed_tasks:    value.failed() as u32,
			total_items:     1,
			succeeded_items: value.success() as u64,
			failed_items:    value.failed() as u64,
			percent:         value.percent().map(Into::into),
		}
	}
}

impl Progress for SizeProg {
	fn running(self) -> bool { !self.done }

	fn cooked(self) -> bool { self.done }

	fn failed(self) -> bool { false }
}
