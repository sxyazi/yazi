use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::tab::Tab;

impl Tab {
	pub fn linemode(&mut self, mut c: CmdCow) {
		render!(self.pref.patch(|new| {
			let Some(mode) = c.take_first_str() else {
				return;
			};
			if !mode.is_empty() && mode.len() <= 20 {
				new.linemode = mode.into_owned();
			}
		}));
	}
}
