use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::input::DeleteOpt;
use yazi_shared::data::Data;

use crate::input::{Input, op::InputOp};

impl Input {
	pub fn delete(&mut self, opt: DeleteOpt) -> Result<Data> {
		match self.snap().op {
			InputOp::None => {
				self.snap_mut().op = InputOp::Delete(opt.cut, opt.insert, self.snap().cursor);
			}
			InputOp::Select(start) => {
				self.snap_mut().op = InputOp::Delete(opt.cut, opt.insert, start);
				render!(self.handle_op(self.snap().cursor, true));
				act!(r#move, self)?;
			}
			InputOp::Delete(..) => {
				self.snap_mut().op = InputOp::Delete(opt.cut, opt.insert, 0);
				act!(r#move, self, self.snap().len() as isize)?;
			}
			_ => {}
		}
		succ!();
	}
}
