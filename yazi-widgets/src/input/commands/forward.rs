use yazi_shared::{CharKind, event::CmdCow};

use crate::input::{Input, op::InputOp};

struct Opt {
	far:         bool,
	end_of_word: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { far: c.bool("far"), end_of_word: c.bool("end-of-word") } }
}

impl Input {
	#[yazi_codegen::command]
	pub fn forward(&mut self, opt: Opt) {
		let snap = self.snap();

		let mut it = snap.value.chars().skip(snap.cursor).enumerate();
		let Some(mut prev) = it.next().map(|(_, c)| CharKind::new(c)) else {
			return self.r#move(0);
		};

		for (i, c) in it {
			let k = CharKind::new(c);
			let b = if opt.end_of_word {
				prev != CharKind::Space && prev.vary(k, opt.far) && i != 1
			} else {
				k != CharKind::Space && k.vary(prev, opt.far)
			};
			if b && !matches!(snap.op, InputOp::None | InputOp::Select(_)) {
				return self.r#move(i as isize);
			} else if b {
				return self.r#move(if opt.end_of_word { i - snap.mode.delta() } else { i } as isize);
			}
			prev = k;
		}

		self.r#move(snap.len() as isize)
	}
}
