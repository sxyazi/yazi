use serde::Serialize;

use crate::{CleanupState, Progress, TaskSummary};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PreloadProg {
	pub state:   Option<bool>,
	pub cleaned: CleanupState,
}

impl From<PreloadProg> for TaskSummary {
	fn from(value: PreloadProg) -> Self {
		Self {
			total:   1,
			success: value.success() as u32,
			failed:  value.failed() as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for PreloadProg {
	fn running(self) -> bool { self.cooking_or_cleaning(self.state.is_none()) }

	fn cooked(self) -> bool { self.state == Some(true) }

	fn failed(self) -> bool { self.cleaned.is_failed() || self.state == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }
}
