use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;

use crate::input::{Input, InputMode, op::InputOp};

impl Input {
	pub fn escape(&mut self, _: VoidOpt) -> Result<Data> {
		let snap = self.snap_mut();
		match snap.mode {
			InputMode::Normal => {
				snap.op = InputOp::None;
			}
			InputMode::Insert => {
				snap.mode = InputMode::Normal;
				act!(r#move, self, -1)?;
			}
			InputMode::Replace => {
				snap.mode = InputMode::Normal;
			}
		}
		self.snaps.tag(self.limit);
		succ!();
	}
}
