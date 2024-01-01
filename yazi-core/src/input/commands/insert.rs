use yazi_shared::{event::Exec, render};

use crate::input::{op::InputOp, Input, InputMode};

pub struct Opt {
	append: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { append: e.named.contains_key("append") } }
}
impl From<bool> for Opt {
	fn from(append: bool) -> Self { Self { append } }
}

impl Input {
	pub fn insert(&mut self, opt: impl Into<Opt>) {
		let snap = self.snap_mut();
		if snap.mode == InputMode::Normal {
			snap.op = InputOp::None;
			snap.mode = InputMode::Insert;
		} else {
			return;
		}

		let opt = opt.into() as Opt;
		if opt.append {
			self.move_(1);
		}

		render!();
	}
}
