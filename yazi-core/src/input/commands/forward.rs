use yazi_shared::{CharKind, event::Cmd};

use crate::input::{Input, op::InputOp};

struct Opt {
	end_of_word: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { end_of_word: c.bool("end-of-word") } }
}

impl Input {
	#[yazi_codegen::command]
	pub fn forward(&mut self, opt: Opt) {
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
