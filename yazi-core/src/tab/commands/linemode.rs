use yazi_macro::render;
use yazi_shared::event::Cmd;

use crate::tab::Tab;

impl Tab {
	pub fn linemode(&mut self, mut c: Cmd) {
		render!(self.conf.patch(|new| {
			let Some(mode) = c.take_first_str() else {
				return;
			};
			if !mode.is_empty() && mode.len() <= 20 {
				new.linemode = mode;
			}
		}));
	}
}
