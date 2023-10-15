use std::mem;

use crate::{emit, tab::Tab};

impl Tab {
	pub fn enter(&mut self) -> bool {
		let Some(hovered) = self.current.hovered().filter(|h| h.is_dir()).map(|h| h.url()) else {
			return false;
		};

		// Current
		let rep = self.history_new(&hovered);
		let rep = mem::replace(&mut self.current, rep);
		if rep.cwd.is_regular() {
			self.history.insert(rep.cwd.clone(), rep);
		}

		// Parent
		if let Some(rep) = self.parent.take() {
			self.history.insert(rep.cwd.clone(), rep);
		}
		self.parent = Some(self.history_new(&hovered.parent_url().unwrap()));

		// Backstack
		self.backstack.push(hovered);

		emit!(Refresh);
		true
	}
}
