use yazi_shared::{event::Exec, CharKind};

use crate::input::Input;

impl Input {
	pub fn backward(&mut self, _: &Exec) {
		let snap = self.snap();
		if snap.cursor == 0 {
			return self.move_(0);
		}

		let idx = snap.idx(snap.cursor).unwrap_or(snap.len());
		let mut it = snap.value[..idx].chars().rev().enumerate();
		let mut prev = CharKind::new(it.next().unwrap().1);
		for (i, c) in it {
			let c = CharKind::new(c);
			if prev != CharKind::Space && prev != c {
				return self.move_(-(i as isize));
			}
			prev = c;
		}

		if prev != CharKind::Space {
			self.move_(-(snap.len() as isize));
		}
	}
}
