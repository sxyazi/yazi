use anyhow::Result;
use yazi_macro::act;
use yazi_shared::data::Data;

use crate::input::{CharKind, Input, parser::BackwardOpt};

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
			if prev != CharKind::Space && prev.vary(k, opt.gait) {
				return act!(r#move, self, -(i as isize));
			}
			prev = k;
		}

		act!(r#move, self, -(snap.len() as isize))
	}
}
