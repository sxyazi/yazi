use yazi_shared::{event::Cmd, render};

use crate::input::{Input, op::InputOp};

pub struct Opt {
	cut:    bool,
	insert: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { cut: c.bool("cut"), insert: c.bool("insert") } }
}

impl Input {
	pub fn delete(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		match self.snap().op {
			InputOp::None => {
				self.snap_mut().op = InputOp::Delete(opt.cut, opt.insert, self.snap().cursor);
			}
			InputOp::Select(start) => {
				self.snap_mut().op = InputOp::Delete(opt.cut, opt.insert, start);
				render!(self.handle_op(self.snap().cursor, true));
				self.move_(0);
			}
			InputOp::Delete(..) => {
				self.snap_mut().op = InputOp::Delete(opt.cut, opt.insert, 0);
				self.move_(self.snap().len() as isize);
			}
			_ => {}
		}
	}
}
