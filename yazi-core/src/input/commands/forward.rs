use yazi_config::keymap::Exec;
use yazi_shared::CharKind;

use crate::input::{op::InputOp, Input};

pub struct Opt {
	end_of_word: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { end_of_word: e.named.contains_key("end-of-word") } }
}

impl Input {
	pub fn forward(&mut self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into() as Opt;
		let snap = self.snap();

		let mut it = snap.value.chars().skip(snap.cursor).enumerate();
		let Some(mut prev) = it.next().map(|(_, c)| CharKind::new(c)) else {
			return self.move_(0);
		};

		for (i, c) in it {
			let c = CharKind::new(c);
			let b = if opt.end_of_word {
				prev != CharKind::Space && prev != c && i != 1
			} else {
				c != CharKind::Space && c != prev
			};
			if b && !matches!(snap.op, InputOp::None | InputOp::Select(_)) {
				return self.move_(i as isize);
			} else if b {
				return self.move_(if opt.end_of_word { i - 1 } else { i } as isize);
			}
			prev = c;
		}

		self.move_(snap.len() as isize)
	}
}
