use serde::Serialize;

use crate::TaskSummary;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PreloadProg {
	pub state:   Option<bool>,
	pub cleaned: Option<bool>,
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

impl PreloadProg {
	pub fn cooked(self) -> bool { self.state == Some(true) }

	pub fn running(self) -> bool { self.state.is_none() || (self.cleaned.is_none() && self.cooked()) }

	pub fn success(self) -> bool { self.cleaned == Some(true) && self.cooked() }

	pub fn failed(self) -> bool { self.cleaned == Some(false) || self.state == Some(false) }

	pub fn cleaned(self) -> Option<bool> { self.cleaned }

	pub fn percent(self) -> Option<f32> { None }
}
