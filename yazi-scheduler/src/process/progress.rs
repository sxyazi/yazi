use serde::Serialize;
use yazi_parser::app::TaskSummary;

// --- Block
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProcessProgBlock {
	pub state: Option<bool>,
}

impl From<ProcessProgBlock> for TaskSummary {
	fn from(value: ProcessProgBlock) -> Self {
		Self {
			total:   (value.state == Some(false)) as u32,
			success: 0,
			failed:  (value.state == Some(false)) as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl ProcessProgBlock {
	pub fn cooked(self) -> bool { self.state == Some(true) }

	pub fn running(self) -> bool { self.state.is_none() }

	pub fn success(self) -> bool { self.cooked() }

	pub fn failed(self) -> bool { self.state == Some(false) }

	pub fn cleaned(self) -> Option<bool> { None }

	pub fn percent(self) -> Option<f32> { None }
}

// --- Orphan
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProcessProgOrphan {
	pub state: Option<bool>,
}

impl From<ProcessProgOrphan> for TaskSummary {
	fn from(value: ProcessProgOrphan) -> Self {
		Self {
			total:   (value.state == Some(false)) as u32,
			success: 0,
			failed:  (value.state == Some(false)) as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl ProcessProgOrphan {
	pub fn cooked(self) -> bool { self.state == Some(true) }

	pub fn running(self) -> bool { self.state.is_none() }

	pub fn success(self) -> bool { self.cooked() }

	pub fn failed(self) -> bool { self.state == Some(false) }

	pub fn cleaned(self) -> Option<bool> { None }

	pub fn percent(self) -> Option<f32> { None }
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
			success: (value.state == Some(true)) as u32,
			failed:  (value.state == Some(false)) as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl ProcessProgBg {
	pub fn cooked(self) -> bool { self.state == Some(true) }

	pub fn running(self) -> bool { self.state.is_none() }

	pub fn success(self) -> bool { self.cooked() }

	pub fn failed(self) -> bool { self.state == Some(false) }

	pub fn cleaned(self) -> Option<bool> { None }

	pub fn percent(self) -> Option<f32> { None }
}
