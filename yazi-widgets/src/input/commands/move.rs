use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::input::MoveOpt;
use yazi_shared::data::Data;

use crate::input::{Input, op::InputOp, snap::InputSnap};

impl Input {
	pub fn r#move(&mut self, opt: MoveOpt) -> Result<Data> {
		let snap = self.snap();
		if opt.in_operating && snap.op == InputOp::None {
			succ!();
		}

		let o_cur = snap.cursor;
		render!(self.handle_op(opt.step.add(&snap.value, snap.cursor), false));
		let n_cur = self.snap().cursor;

		let (limit, snap) = (self.limit, self.snap_mut());
		if snap.value.is_empty() {
			succ!(snap.offset = 0);
		}

		let (o_off, scrolloff) = (snap.offset, 5.min(limit / 2));
		snap.offset = if n_cur <= o_cur {
			let it = snap.slice(0..n_cur).chars().rev().map(|c| if snap.obscure { '•' } else { c });
			let pad = InputSnap::find_window(it, 0, scrolloff).end;

			if n_cur >= o_off { snap.offset.min(n_cur - pad) } else { n_cur - pad }
		} else {
			let count = snap.count();

			let it = snap.slice(n_cur..count).chars().map(|c| if snap.obscure { '•' } else { c });
			let pad = InputSnap::find_window(it, 0, scrolloff + snap.mode.delta()).end;

			let it = snap.slice(0..n_cur + pad).chars().rev().map(|c| if snap.obscure { '•' } else { c });
			let max = InputSnap::find_window(it, 0, limit).end;

			if snap.width(o_off..n_cur) < limit as u16 {
				snap.offset.max(n_cur + pad - max)
			} else {
				n_cur + pad - max
			}
		};
		succ!();
	}
}
