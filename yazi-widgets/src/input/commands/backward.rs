use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::input::BackwardOpt;
use yazi_shared::{CharKind, data::Data};

use crate::input::Input;

impl Input {
	pub fn backward(&mut self, opt: BackwardOpt) -> Result<Data> {
		let snap = self.snap();
		if snap.cursor == 0 {
			return act!(r#move, self);
		}

		let idx = snap.idx(snap.cursor).unwrap_or(snap.len());
		let mut it = snap.value[..idx].chars().rev().enumerate();
		let mut prev = CharKind::new(it.next().unwrap().1);
		for (i, c) in it {
			let k = CharKind::new(c);
			if prev != CharKind::Space && prev.vary(k, opt.far) {
				return act!(r#move, self, -(i as isize));
			}
			prev = k;
		}

		if prev != CharKind::Space {
			act!(r#move, self, -(snap.len() as isize))?;
		}
		succ!();
	}
}
