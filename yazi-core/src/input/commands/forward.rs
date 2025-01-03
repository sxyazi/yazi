use yazi_shared::{CharKind, event::CmdCow};

use crate::input::{Input, op::InputOp};

struct Opt {
	end_of_word: bool,
	big: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self {
			end_of_word: c.bool("end-of-word"),
			big:         c.bool("big"),
		}
	}
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
			let new_char_kind = if opt.big {
				(c == CharKind::Space) != (prev == CharKind::Space)
			} else {
				c != prev
			};
			let b = if opt.end_of_word {
				prev != CharKind::Space && new_char_kind && i != 1
			} else {
				c != CharKind::Space && new_char_kind
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
