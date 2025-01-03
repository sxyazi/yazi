use yazi_shared::{CharKind, event::CmdCow};

use crate::input::Input;

struct Opt {
	big: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { big: c.bool("big") }
	}
}

impl Input {
	#[yazi_codegen::command]
	pub fn backward(&mut self, opt: Opt) {
		let snap = self.snap();
		if snap.cursor == 0 {
			return self.move_(0);
		}

		let idx = snap.idx(snap.cursor).unwrap_or(snap.len());
		let mut it = snap.value[..idx].chars().rev().enumerate();
		let mut prev = CharKind::new(it.next().unwrap().1);
		for (i, c) in it {
			let c = CharKind::new(c);
			let new_char_kind = if opt.big {
				(c == CharKind::Space) != (prev == CharKind::Space)
			} else {
				c != prev
			};
			if prev != CharKind::Space && new_char_kind {
				return self.move_(-(i as isize));
			}
			prev = c;
		}

		if prev != CharKind::Space {
			self.move_(-(snap.len() as isize));
		}
	}
}
