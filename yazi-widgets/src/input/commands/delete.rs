use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::input::{Input, op::InputOp};

struct Opt {
	cut:    bool,
	insert: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { cut: c.bool("cut"), insert: c.bool("insert") } }
}

impl Input {
	#[yazi_codegen::command]
	pub fn delete(&mut self, opt: Opt) {
		match self.snap().op {
			InputOp::None => {
				self.snap_mut().op = InputOp::Delete(opt.cut, opt.insert, self.snap().cursor);
			}
			InputOp::Select(start) => {
				self.snap_mut().op = InputOp::Delete(opt.cut, opt.insert, start);
				render!(self.handle_op(self.snap().cursor, true));
				self.r#move(0);
			}
			InputOp::Delete(..) => {
				self.snap_mut().op = InputOp::Delete(opt.cut, opt.insert, 0);
				self.r#move(self.snap().len() as isize);
			}
			_ => {}
		}
	}
}
