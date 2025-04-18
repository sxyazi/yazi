use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::tab::Tab;

impl Tab {
	pub fn linemode(&mut self, mut c: CmdCow) {
		let Some(new) = c.take_first_str() else { return };
		if new == self.pref.linemode {
			return;
		} else if new.is_empty() || new.len() > 20 {
			return;
		}

		self.pref.linemode = new.into_owned();
		render!();
	}
}
