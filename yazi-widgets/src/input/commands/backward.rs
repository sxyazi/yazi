use yazi_shared::{CharKind, event::CmdCow};

use crate::input::Input;

struct Opt {
	far: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { far: c.bool("far") } }
}

impl Input {
	#[yazi_codegen::command]
	pub fn backward(&mut self, opt: Opt) {
		let snap = self.snap();
		if snap.cursor == 0 {
			return self.r#move(0);
		}

		let idx = snap.idx(snap.cursor).unwrap_or(snap.len());
		let mut it = snap.value[..idx].chars().rev().enumerate();
		let mut prev = CharKind::new(it.next().unwrap().1);
		for (i, c) in it {
			let k = CharKind::new(c);
			if prev != CharKind::Space && prev.vary(k, opt.far) {
				return self.r#move(-(i as isize));
			}
			prev = k;
		}

		if prev != CharKind::Space {
			self.r#move(-(snap.len() as isize));
		}
	}
}
