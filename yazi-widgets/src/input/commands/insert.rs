use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::input::{Input, InputMode, op::InputOp};

struct Opt {
	append: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { append: c.bool("append") } }
}
impl From<bool> for Opt {
	fn from(append: bool) -> Self { Self { append } }
}

impl Input {
	#[yazi_codegen::command]
	pub fn insert(&mut self, opt: Opt) {
		let snap = self.snap_mut();
		if snap.mode == InputMode::Normal {
			snap.op = InputOp::None;
			snap.mode = InputMode::Insert;
		} else {
			return;
		}

		if opt.append {
			self.r#move(1);
		}

		render!();
	}
}
