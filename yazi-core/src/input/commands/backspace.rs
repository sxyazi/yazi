use yazi_shared::event::Exec;

use crate::input::Input;

pub struct Opt {
	under: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { under: e.named.contains_key("under") } }
}
impl From<bool> for Opt {
	fn from(under: bool) -> Self { Self { under } }
}

impl Input {
	pub fn backspace(&mut self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into() as Opt;
		let snap = self.snaps.current_mut();

		if !opt.under && snap.cursor < 1 {
			return false;
		} else if opt.under && snap.cursor >= snap.value.len() {
			return false;
		}

		if opt.under {
			snap.value.remove(snap.idx(snap.cursor).unwrap());
			self.move_(0);
		} else {
			snap.value.remove(snap.idx(snap.cursor - 1).unwrap());
			self.move_(-1);
		}

		self.flush_value();
		true
	}
}
