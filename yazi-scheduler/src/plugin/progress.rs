use serde::Serialize;

use crate::{Progress, TaskSummary};

// --- Entry
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PluginProgEntry {
	pub(crate) state: Option<bool>,
}

impl From<PluginProgEntry> for TaskSummary {
	fn from(value: PluginProgEntry) -> Self {
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

impl Progress for PluginProgEntry {
	fn running(self) -> bool { self.state.is_none() }

	fn cooked(self) -> bool { self.state == Some(true) }

	fn failed(self) -> bool { self.state == Some(false) }
}
