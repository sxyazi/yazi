use serde::Serialize;

use crate::{CleanupState, Progress, TaskSummary};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FetchProg {
	pub(crate) state:   Option<bool>,
	pub(crate) cleaned: CleanupState,
}

impl From<FetchProg> for TaskSummary {
	fn from(value: FetchProg) -> Self {
		Self {
			total_tasks:     1,
			succeeded_tasks: value.success() as u32,
			failed_tasks:    value.failed() as u32,
			total_items:     1,
			succeeded_items: value.cooked() as u64,
			failed_items:    (value.state == Some(false)) as u64,
			percent:         value.percent().map(Into::into),
		}
	}
}

impl Progress for FetchProg {
	fn running(self) -> bool { self.cooking_or_cleaning(self.state.is_none()) }

	fn cooked(self) -> bool { self.state == Some(true) }

	fn failed(self) -> bool { self.cleaned.is_failed() || self.state == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }
}
