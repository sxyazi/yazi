use yazi_config::keymap::Exec;

use crate::input::{op::InputOp, Input};

pub struct Opt {
	end_of_word: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { end_of_word: e.named.contains_key("end-of-word") } }
}
impl From<bool> for Opt {
	fn from(end_of_word: bool) -> Self { Self { end_of_word } }
}

impl Input {
	pub fn forward(&mut self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into() as Opt;

		let snap = self.snap();
		let idx = snap.idx(snap.cursor).unwrap_or(snap.len());

		let step = Self::find_word_boundary(
			snap.value[idx..].chars(),
			// When in vim mode and deleting, we want to skip over the boundary of words.
			// In other words |A|bcd efg when deleting till the end of the word should be
			// | |efg as opposed to |d| efg.
			if let InputOp::Delete(..) = snap.op { false } else { opt.end_of_word },
			opt.end_of_word,
		);
		self.move_(step as isize)
	}
}
