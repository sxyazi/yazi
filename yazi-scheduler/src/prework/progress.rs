use yazi_parser::app::TaskSummary;

// --- Fetch
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PreworkProgFetch {
	pub(crate) state: Option<bool>,
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
	pub(crate) fn running(self) -> bool { self.state.is_none() }

	pub(crate) fn success(self) -> bool { self.state == Some(true) }

	pub(crate) fn percent(self) -> Option<f32> { None }
}

// --- Load
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PreworkProgLoad {
	pub(crate) state: Option<bool>,
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
	pub(crate) fn running(self) -> bool { self.state.is_none() }

	pub(crate) fn success(self) -> bool { self.state == Some(true) }

	pub(crate) fn percent(self) -> Option<f32> { None }
}

// --- Size
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PreworkProgSize {
	pub(crate) done: bool,
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
	pub(crate) fn running(self) -> bool { !self.done }

	pub(crate) fn success(self) -> bool { self.done }

	pub(crate) fn percent(self) -> Option<f32> { None }
}
