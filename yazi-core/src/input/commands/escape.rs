use yazi_macro::render;
use yazi_proxy::CmpProxy;
use yazi_shared::event::CmdCow;
use yazi_widgets::input::InputOp;

use crate::input::Input;

struct Opt;

impl From<CmdCow> for Opt {
	fn from(_: CmdCow) -> Self { Self }
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Input {
	#[yazi_codegen::command]
	pub fn escape(&mut self, _: Opt) {
		use yazi_widgets::input::InputMode as M;

		let mode = self.snap().mode;
		match mode {
			M::Normal if self.snap_mut().op == InputOp::None => self.close(false),
			M::Insert => CmpProxy::close(),
			M::Normal | M::Replace => {}
		}

		self.inner.escape(());
		render!();
	}
}
