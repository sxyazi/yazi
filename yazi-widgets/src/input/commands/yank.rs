use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;

use crate::input::{Input, op::InputOp};

impl Input {
	pub fn yank(&mut self, _: VoidOpt) -> Result<Data> {
		match self.snap().op {
			InputOp::None => {
				self.snap_mut().op = InputOp::Yank(self.snap().cursor);
			}
			InputOp::Select(start) => {
				self.snap_mut().op = InputOp::Yank(start);
				render!(self.handle_op(self.snap().cursor, true));
				act!(r#move, self)?;
			}
			InputOp::Yank(_) => {
				self.snap_mut().op = InputOp::Yank(0);
				act!(r#move, self, self.snap().len() as isize)?;
			}
			_ => {}
		}
		succ!();
	}
}
