use yazi_shared::event::Exec;

use crate::input::{op::InputOp, Input};

pub struct Opt {
	cut:    bool,
	insert: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self { cut: e.named.contains_key("cut"), insert: e.named.contains_key("insert") }
	}
}

impl Input {
	pub fn delete(&mut self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into() as Opt;
		match self.snap().op {
			InputOp::None => {
				self.snap_mut().op = InputOp::Delete(opt.cut, opt.insert, self.snap().cursor);
				false
			}
			InputOp::Select(start) => {
				self.snap_mut().op = InputOp::Delete(opt.cut, opt.insert, start);
				return self.handle_op(self.snap().cursor, true).then(|| self.move_(0)).is_some();
			}
			InputOp::Delete(..) => {
				self.snap_mut().op = InputOp::Delete(opt.cut, opt.insert, 0);
				return self.move_(self.snap().len() as isize);
			}
			_ => false,
		}
	}
}
