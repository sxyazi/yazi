use yazi_config::popup::{Offset, Origin, Position};
use yazi_shared::Exec;

use crate::{help::Help, input::Input};

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Help {
	pub fn filter(&mut self, _: impl Into<Opt>) -> bool {
		let mut input = Input::default();
		input.position = Position::new(Origin::BottomLeft, Offset::line());

		self.in_filter = Some(input);
		self.filter_apply();
		true
	}
}
