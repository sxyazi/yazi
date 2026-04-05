use serde::Serialize;

use crate::{Progress, TaskSummary};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct SizeProg {
	pub done: bool,
}

impl From<SizeProg> for TaskSummary {
	fn from(value: SizeProg) -> Self {
		Self {
			total:   1,
			success: value.done as u32,
			failed:  0,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for SizeProg {
	fn running(self) -> bool { !self.done }

	fn cooked(self) -> bool { self.done }

	fn failed(self) -> bool { false }
}
