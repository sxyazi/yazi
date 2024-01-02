use yazi_shared::{event::Exec, render};

use crate::input::{op::InputOp, Input};

impl Input {
	pub fn yank(&mut self, _: &Exec) {
		match self.snap().op {
			InputOp::None => {
				self.snap_mut().op = InputOp::Yank(self.snap().cursor);
			}
			InputOp::Select(start) => {
				self.snap_mut().op = InputOp::Yank(start);
				render!(self.handle_op(self.snap().cursor, true));
				self.move_(0);
			}
			InputOp::Yank(_) => {
				self.snap_mut().op = InputOp::Yank(0);
				self.move_(self.snap().len() as isize);
			}
			_ => {}
		}
	}
}
