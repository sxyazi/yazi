use yazi_config::keymap::Key;
use yazi_shared::event::CmdCow;

use crate::input::{Input, InputMode};

struct Opt;

impl From<CmdCow> for Opt {
	fn from(_: CmdCow) -> Self { Self }
}

impl Input {
	pub fn type_(&mut self, key: &Key) -> bool {
		if self.mode() == InputMode::Replace {
			let Some(c) = key.plain() else {
				return false;
			};

			let snap = self.snaps.current_mut();
			snap.mode = InputMode::Normal;
			self.replace_str(c.encode_utf8(&mut [0; 4]));

			return true;
		}

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
