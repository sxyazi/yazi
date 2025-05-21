use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::input::Input;

struct Opt {
	under: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { under: c.bool("under") } }
}
impl From<bool> for Opt {
	fn from(under: bool) -> Self { Self { under } }
}

impl Input {
	#[yazi_codegen::command]
	pub fn backspace(&mut self, opt: Opt) {
		let snap = self.snap_mut();
		if !opt.under && snap.cursor < 1 {
			return;
		} else if opt.under && snap.cursor >= snap.count() {
			return;
		}

		if opt.under {
			snap.value.remove(snap.idx(snap.cursor).unwrap());
			self.r#move(0);
		} else {
			snap.value.remove(snap.idx(snap.cursor - 1).unwrap());
			self.r#move(-1);
		}

		self.flush_value();
		render!();
	}
}
