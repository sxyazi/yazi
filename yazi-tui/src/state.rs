use crate::RatermOption;

#[derive(Clone, Copy)]
pub struct RatermState {
	pub mouse:   bool,
	pub title:   bool,
	pub started: bool,
}

impl RatermState {
	pub(super) const fn default() -> Self { Self { mouse: false, title: false, started: false } }

	pub(super) fn new(opt: &RatermOption) -> Self {
		Self { mouse: opt.mouse, started: true, ..Self::default() }
	}
}
