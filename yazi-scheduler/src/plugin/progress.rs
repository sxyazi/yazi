use serde::Serialize;
use yazi_parser::app::TaskSummary;

// --- Entry
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PluginProgEntry {
	pub state: Option<bool>,
}

impl From<PluginProgEntry> for TaskSummary {
	fn from(value: PluginProgEntry) -> Self {
		Self {
			total:   1,
			success: (value.state == Some(true)) as u32,
			failed:  (value.state == Some(false)) as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl PluginProgEntry {
	pub fn running(self) -> bool { self.state.is_none() }

	pub fn success(self) -> bool { self.state == Some(true) }

	pub fn cleaned(self) -> bool { false }

	pub fn percent(self) -> Option<f32> { None }
}
