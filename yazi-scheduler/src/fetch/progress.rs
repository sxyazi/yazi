use serde::Serialize;

use crate::{Progress, TaskSummary};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FetchProg {
	pub state: Option<bool>,
}

impl From<FetchProg> for TaskSummary {
	fn from(value: FetchProg) -> Self {
		Self {
			total:   1,
			success: value.success() as u32,
			failed:  value.failed() as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for FetchProg {
	fn running(self) -> bool { self.state.is_none() }

	fn cooked(self) -> bool { self.state == Some(true) }

	fn failed(self) -> bool { self.state == Some(false) }
}
