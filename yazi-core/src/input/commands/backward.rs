use yazi_config::keymap::Exec;

use crate::input::Input;

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Input {
	pub fn backward(&mut self, _: impl Into<Opt>) -> bool {
		let snap = self.snap();
		let idx = snap.idx(snap.cursor).unwrap_or(snap.len());

		let step = Self::find_word_boundary(snap.value[..idx].chars().rev(), false);
		self.move_(-(step as isize))
	}
}
