use yazi_macro::render;
use yazi_plugin::CLIPBOARD;
use yazi_shared::event::CmdCow;

use crate::input::{Input, op::InputOp};

struct Opt {
	before: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { before: c.bool("before") } }
}

impl Input {
	#[yazi_codegen::command]
	pub fn paste(&mut self, opt: Opt) {
		if let Some(start) = self.snap().op.start() {
			self.snap_mut().op = InputOp::Delete(false, false, start);
			self.handle_op(self.snap().cursor, true);
		}

		let s = futures::executor::block_on(CLIPBOARD.get());
		if s.is_empty() {
			return;
		}

		self.insert(!opt.before);
		self.type_str(&s.to_string_lossy());
		self.escape(());
		render!();
	}
}
