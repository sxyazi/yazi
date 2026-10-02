use super::{State, Width};

pub(super) struct TimeParser {
	pub(super) state:  State,
	pub(super) widths: [Width; 3],
}

impl TimeParser {
	pub(super) fn new() -> Self { Self { state: State::Pattern, widths: [Width::Numeric; 3] } }
}
