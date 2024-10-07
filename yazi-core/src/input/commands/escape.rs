use yazi_proxy::CompletionProxy;
use yazi_shared::{event::Cmd, render};

use crate::input::{Input, InputMode, op::InputOp};

pub struct Opt;

impl From<Cmd> for Opt {
	fn from(_: Cmd) -> Self { Self }
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Input {
	#[yazi_macro::command]
	pub fn escape(&mut self, _: Opt) {
		let snap = self.snap_mut();
		match snap.mode {
			InputMode::Normal if snap.op == InputOp::None => {
				self.close(false);
			}
			InputMode::Normal => {
				snap.op = InputOp::None;
			}
			InputMode::Insert => {
				snap.mode = InputMode::Normal;
				self.move_(-1);

				if self.completion {
					CompletionProxy::close();
				}
			}
		}

		self.snaps.tag(self.limit());
		render!();
	}
}
