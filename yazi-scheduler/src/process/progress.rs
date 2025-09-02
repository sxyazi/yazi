use yazi_parser::app::TaskSummary;

// --- Block
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ProcessProgBlock {
	pub(crate) state: Option<bool>,
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
	pub(crate) fn running(self) -> bool { self.state.is_none() }

	pub(crate) fn success(self) -> bool { self.state == Some(true) }

	pub(crate) fn percent(self) -> Option<f32> { None }
}

// --- Orphan
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ProcessProgOrphan {
	pub(crate) state: Option<bool>,
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
	pub(crate) fn running(self) -> bool { self.state.is_none() }

	pub(crate) fn success(self) -> bool { self.state == Some(true) }

	pub(crate) fn percent(self) -> Option<f32> { None }
}

// --- Bg
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ProcessProgBg {
	pub(crate) state: Option<bool>,
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
	pub(crate) fn running(self) -> bool { self.state.is_none() }

	pub(crate) fn success(self) -> bool { self.state == Some(true) }

	pub(crate) fn percent(self) -> Option<f32> { None }
}
