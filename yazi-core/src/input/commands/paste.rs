use yazi_plugin::CLIPBOARD;
use yazi_shared::{event::Cmd, render};

use crate::input::{op::InputOp, Input};

pub struct Opt {
	before: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { before: c.bool("before") } }
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
