use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::input::{Input, op::InputOp};

impl Input {
	pub fn yank(&mut self, _: CmdCow) {
		match self.snap().op {
			InputOp::None => {
				self.snap_mut().op = InputOp::Yank(self.snap().cursor);
			}
			InputOp::Select(start) => {
				self.snap_mut().op = InputOp::Yank(start);
				render!(self.handle_op(self.snap().cursor, true));
				self.r#move(0);
			}
			InputOp::Yank(_) => {
				self.snap_mut().op = InputOp::Yank(0);
				self.r#move(self.snap().len() as isize);
			}
			_ => {}
		}
	}
}
