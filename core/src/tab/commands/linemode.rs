use config::keymap::Exec;

use crate::tab::Tab;

impl Tab {
	pub fn linemode(&mut self, e: &Exec) -> bool {
		self.conf.patch(|c| {
			let Some(mode) = e.args.get(0) else {
				return;
			};
			if !mode.is_empty() && mode.len() <= 20 {
				c.linemode = mode.to_owned();
			}
		})
	}
}
