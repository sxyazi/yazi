use yazi_config::keymap::Exec;

use crate::input::Input;

pub struct Opt {
	append: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { append: e.named.contains_key("append") } }
}
impl From<bool> for Opt {
	fn from(append: bool) -> Self { Self { append } }
}

impl Input {
	pub fn insert(&mut self, opt: impl Into<Opt>) -> bool {
		if !self.snap_mut().insert() {
			return false;
		}

		let opt = opt.into() as Opt;
		if opt.append {
			self.move_(1);
		}

		true
	}
}
