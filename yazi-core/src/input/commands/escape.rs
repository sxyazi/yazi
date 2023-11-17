use yazi_config::keymap::{Exec, KeymapLayer};

use crate::{emit, input::{op::InputOp, Input, InputMode}};

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Input {
	pub fn escape(&mut self, _: impl Into<Opt>) -> bool {
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
					emit!(Call(Exec::call("close", vec![]).vec(), KeymapLayer::Completion));
				}
			}
		}
		self.snaps.tag(self.limit());
		true
	}
}
