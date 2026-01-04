use serde::Serialize;
use yazi_parser::app::TaskSummary;

// --- Fetch
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PreworkProgFetch {
	pub state: Option<bool>,
}

impl From<PreworkProgFetch> for TaskSummary {
	fn from(value: PreworkProgFetch) -> Self {
		Self {
			total:   1,
			success: (value.state == Some(true)) as u32,
			failed:  (value.state == Some(false)) as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl PreworkProgFetch {
	pub fn cooked(self) -> bool { self.state == Some(true) }

	pub fn running(self) -> bool { self.state.is_none() }

	pub fn success(self) -> bool { self.cooked() }

	pub fn failed(self) -> bool { self.state == Some(false) }

	pub fn cleaned(self) -> Option<bool> { None }

	pub fn percent(self) -> Option<f32> { None }
}

// --- Load
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PreworkProgLoad {
	pub state: Option<bool>,
}

impl From<PreworkProgLoad> for TaskSummary {
	fn from(value: PreworkProgLoad) -> Self {
		Self {
			total:   1,
			success: (value.state == Some(true)) as u32,
			failed:  (value.state == Some(false)) as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl PreworkProgLoad {
	pub fn cooked(self) -> bool { self.state == Some(true) }

	pub fn running(self) -> bool { self.state.is_none() }

	pub fn success(self) -> bool { self.cooked() }

	pub fn failed(self) -> bool { self.state == Some(false) }

	pub fn cleaned(self) -> Option<bool> { None }

	pub fn percent(self) -> Option<f32> { None }
}

// --- Size
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PreworkProgSize {
	pub done: bool,
}

impl From<PreworkProgSize> for TaskSummary {
	fn from(value: PreworkProgSize) -> Self {
		Self {
			total:   1,
			success: value.done as u32,
			failed:  0,
			percent: value.percent().map(Into::into),
		}
	}
}

impl PreworkProgSize {
	pub fn cooked(self) -> bool { self.done }

	pub fn running(self) -> bool { !self.done }

	pub fn success(self) -> bool { self.cooked() }

	pub fn failed(self) -> bool { false }

	pub fn cleaned(self) -> Option<bool> { None }

	pub fn percent(self) -> Option<f32> { None }
}
