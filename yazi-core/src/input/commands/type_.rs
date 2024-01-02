use yazi_config::keymap::Key;
use yazi_shared::event::Exec;

use crate::input::{Input, InputMode};

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Input {
	pub fn type_(&mut self, key: &Key) -> bool {
		if self.mode() != InputMode::Insert {
			return false;
		}

		if let Some(c) = key.plain() {
			self.type_str(c.encode_utf8(&mut [0; 4]));
			return true;
		}

		false
	}
}
