use serde::Serialize;

use crate::{Progress, TaskSummary};

// --- Entry
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PluginProgEntry {
	pub state: Option<bool>,
}

impl From<PluginProgEntry> for TaskSummary {
	fn from(value: PluginProgEntry) -> Self {
		Self {
			total:   1,
			success: value.success() as u32,
			failed:  value.failed() as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for PluginProgEntry {
	fn running(self) -> bool { self.state.is_none() }

	fn cooked(self) -> bool { self.state == Some(true) }

	fn failed(self) -> bool { self.state == Some(false) }
}
