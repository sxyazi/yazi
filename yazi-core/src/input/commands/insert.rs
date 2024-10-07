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
	#[yazi_macro::command]
	pub fn insert(&mut self, opt: Opt) {
		let snap = self.snap_mut();
		if snap.mode == InputMode::Normal {
			snap.op = InputOp::None;
			snap.mode = InputMode::Insert;
		} else {
			return;
		}

		if opt.append {
			self.move_(1);
		}

		render!();
	}
}
