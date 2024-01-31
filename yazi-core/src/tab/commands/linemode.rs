use yazi_shared::{event::Cmd, render};

use crate::tab::Tab;

impl Tab {
	pub fn linemode(&mut self, mut c: Cmd) {
		render!(self.conf.patch(|new| {
			let Some(mode) = c.take_first() else {
				return;
			};
			if !mode.is_empty() && mode.len() <= 20 {
				new.linemode = mode;
			}
		}));
	}
}
