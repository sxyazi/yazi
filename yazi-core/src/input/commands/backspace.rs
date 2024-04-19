use yazi_shared::{event::Cmd, render};

use crate::input::Input;

pub struct Opt {
	under: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { under: c.bool("under") } }
}
impl From<bool> for Opt {
	fn from(under: bool) -> Self { Self { under } }
}

impl Input {
	pub fn backspace(&mut self, opt: impl Into<Opt>) {
		let snap = self.snaps.current_mut();
		if snap.value.is_empty() {
			return self.close(false);
		}

		let opt = opt.into() as Opt;
		if !opt.under && snap.cursor < 1 {
			return;
		} else if opt.under && snap.cursor >= snap.value.len() {
			return;
		}

		if opt.under {
			snap.value.remove(snap.idx(snap.cursor).unwrap());
			self.move_(0);
		} else {
			snap.value.remove(snap.idx(snap.cursor - 1).unwrap());
			self.move_(-1);
		}

		self.flush_value();
		render!();
	}
}
