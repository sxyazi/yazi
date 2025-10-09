use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;

use crate::input::{Input, op::InputOp};

impl Input {
	pub fn uppercase(&mut self, _: VoidOpt) -> Result<Data> { self.apply_case(true) }

	pub fn lowercase(&mut self, _: VoidOpt) -> Result<Data> { self.apply_case(false) }

	fn apply_case(&mut self, uppercase: bool) -> Result<Data> {
		match self.snap().op {
			InputOp::Select(start) => {
				self.snap_mut().op = InputOp::Case(uppercase, start);
				render!(self.handle_op(self.snap().cursor, true));
				act!(r#move, self)?;
			}
			_ => {}
		}
		succ!();
	}
}
