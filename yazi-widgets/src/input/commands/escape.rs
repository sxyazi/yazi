use crate::input::{Input, InputMode, op::InputOp};

struct Opt;

impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Input {
	#[yazi_codegen::command]
	pub fn escape(&mut self, _: Opt) {
		let snap = self.snap_mut();
		match snap.mode {
			InputMode::Normal => {
				snap.op = InputOp::None;
			}
			InputMode::Insert => {
				snap.mode = InputMode::Normal;
				self.r#move(-1);
			}
			InputMode::Replace => {
				snap.mode = InputMode::Normal;
			}
		}
		self.snaps.tag(self.limit);
	}
}
