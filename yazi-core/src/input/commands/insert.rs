use yazi_shared::{event::Cmd, render};

use crate::input::{Input, InputMode, op::InputOp};

pub struct Opt {
	append: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { append: c.bool("append") } }
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
