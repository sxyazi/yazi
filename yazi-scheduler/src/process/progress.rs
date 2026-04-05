use serde::Serialize;

use crate::{Progress, TaskSummary};

// --- Block
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProcessProgBlock {
	pub state: Option<bool>,
}

impl From<ProcessProgBlock> for TaskSummary {
	fn from(value: ProcessProgBlock) -> Self {
		Self {
			total:   value.failed() as u32,
			success: 0,
			failed:  value.failed() as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for ProcessProgBlock {
	fn running(self) -> bool { self.state.is_none() }

	fn cooked(self) -> bool { self.state == Some(true) }

	fn failed(self) -> bool { self.state == Some(false) }
}

// --- Orphan
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProcessProgOrphan {
	pub state: Option<bool>,
}

impl From<ProcessProgOrphan> for TaskSummary {
	fn from(value: ProcessProgOrphan) -> Self {
		Self {
			total:   value.failed() as u32,
			success: 0,
			failed:  value.failed() as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for ProcessProgOrphan {
	fn running(self) -> bool { self.state.is_none() }

	fn cooked(self) -> bool { self.state == Some(true) }

	fn failed(self) -> bool { self.state == Some(false) }
}

// --- Bg
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProcessProgBg {
	pub state: Option<bool>,
}

impl From<ProcessProgBg> for TaskSummary {
	fn from(value: ProcessProgBg) -> Self {
		Self {
			total:   1,
			success: value.success() as u32,
			failed:  value.failed() as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for ProcessProgBg {
	fn running(self) -> bool { self.state.is_none() }

	fn cooked(self) -> bool { self.state == Some(true) }

	fn failed(self) -> bool { self.state == Some(false) }
}
