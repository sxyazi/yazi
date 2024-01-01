use yazi_config::keymap::Key;
use yazi_shared::{event::Exec, render};

use crate::input::{Input, InputMode};

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Input {
	pub fn type_(&mut self, key: &Key) {
		if self.mode() != InputMode::Insert {
			return;
		}

		if let Some(c) = key.plain() {
			let mut bits = [0; 4];
			render!(self.type_str(c.encode_utf8(&mut bits)));
		}
	}
}
