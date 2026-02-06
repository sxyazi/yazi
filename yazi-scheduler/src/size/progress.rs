use serde::Serialize;
use yazi_parser::app::TaskSummary;

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

impl SizeProg {
	pub fn cooked(self) -> bool { self.done }

	pub fn running(self) -> bool { !self.done }

	pub fn success(self) -> bool { self.cooked() }

	pub fn failed(self) -> bool { false }

	pub fn cleaned(self) -> Option<bool> { None }

	pub fn percent(self) -> Option<f32> { None }
}
