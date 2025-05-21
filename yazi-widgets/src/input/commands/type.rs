use yazi_config::keymap::Key;
use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::input::{Input, InputMode};

struct Opt;

impl From<CmdCow> for Opt {
	fn from(_: CmdCow) -> Self { Self }
}

impl Input {
	pub fn r#type(&mut self, key: &Key) -> bool {
		let Some(c) = key.plain() else { return false };

		if self.mode() == InputMode::Insert {
			self.type_str(c.encode_utf8(&mut [0; 4]));
			return true;
		} else if self.mode() == InputMode::Replace {
			self.replace_str(c.encode_utf8(&mut [0; 4]));
			return true;
		}

		false
	}

	pub fn type_str(&mut self, s: &str) {
		let snap = self.snap_mut();
		if snap.cursor < 1 {
			snap.value.insert_str(0, s);
		} else {
			snap.value.insert_str(snap.idx(snap.cursor).unwrap(), s);
		}

		self.r#move(s.chars().count() as isize);
		self.flush_value();
		render!();
	}
}
