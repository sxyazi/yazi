use yazi_shared::{event::Exec, render};

use crate::tab::Tab;

impl Tab {
	pub fn linemode(&mut self, e: &Exec) {
		render!(self.conf.patch(|c| {
			let Some(mode) = e.args.first() else {
				return;
			};
			if !mode.is_empty() && mode.len() <= 20 {
				c.linemode = mode.to_owned();
			}
		}));
	}
}
