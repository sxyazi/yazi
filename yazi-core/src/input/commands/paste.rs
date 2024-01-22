use yazi_shared::{event::Exec, render};

use crate::{input::{op::InputOp, Input}, CLIPBOARD};

pub struct Opt {
	before: bool,
}

impl From<Exec> for Opt {
	fn from(e: Exec) -> Self { Self { before: e.named.contains_key("before") } }
}

impl Input {
	pub fn paste(&mut self, opt: impl Into<Opt>) {
		if let Some(start) = self.snap().op.start() {
			self.snap_mut().op = InputOp::Delete(false, false, start);
			self.handle_op(self.snap().cursor, true);
		}

		let s = futures::executor::block_on(CLIPBOARD.get());
		if s.is_empty() {
			return;
		}

		let opt = opt.into() as Opt;
		self.insert(!opt.before);
		self.type_str(&s.to_string_lossy());
		self.escape(());
		render!();
	}
}
