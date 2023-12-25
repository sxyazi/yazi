use yazi_shared::event::Exec;

use crate::input::{op::InputOp, Input};

impl Input {
	pub fn yank(&mut self, _: &Exec) -> bool {
		match self.snap().op {
			InputOp::None => {
				self.snap_mut().op = InputOp::Yank(self.snap().cursor);
				false
			}
			InputOp::Select(start) => {
				self.snap_mut().op = InputOp::Yank(start);
				return self.handle_op(self.snap().cursor, true).then(|| self.move_(0)).is_some();
			}
			InputOp::Yank(_) => {
				self.snap_mut().op = InputOp::Yank(0);
				self.move_(self.snap().len() as isize);
				false
			}
			_ => false,
		}
	}
}
