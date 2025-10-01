use anyhow::Result;
use yazi_macro::act;
use yazi_parser::input::ForwardOpt;
use yazi_shared::{CharKind, data::Data};

use crate::input::{Input, op::InputOp};

impl Input {
	pub fn forward(&mut self, opt: ForwardOpt) -> Result<Data> {
		let snap = self.snap();

		let mut it = snap.value.chars().skip(snap.cursor).enumerate();
		let Some(mut prev) = it.next().map(|(_, c)| CharKind::new(c)) else {
			return act!(r#move, self);
		};

		for (i, c) in it {
			let k = CharKind::new(c);
			let b = if opt.end_of_word {
				prev != CharKind::Space && prev.vary(k, opt.far) && i != 1
			} else {
				k != CharKind::Space && k.vary(prev, opt.far)
			};
			if b && !matches!(snap.op, InputOp::None | InputOp::Select(_)) {
				return act!(r#move, self, i as isize);
			} else if b {
				return act!(
					r#move,
					self,
					if opt.end_of_word { i - snap.mode.delta() } else { i } as isize
				);
			}
			prev = k;
		}

		act!(r#move, self, snap.len() as isize)
	}
}
