use std::mem;

use yazi_shared::{event::Cmd, render};

use crate::{manager::Manager, tab::Tab};

pub struct Opt;
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}
impl From<Cmd> for Opt {
	fn from(_: Cmd) -> Self { Self }
}

impl Tab {
	pub fn leave(&mut self, _: impl Into<Opt>) {
		let current = self
			.current
			.hovered()
			.and_then(|h| h.parent())
			.filter(|p| *p != self.current.cwd)
			.or_else(|| self.current.cwd.parent_url());

		let Some(current) = current else {
			return;
		};

		// Parent
		if let Some(rep) = self.parent.take() {
			self.history.insert(rep.cwd.clone(), rep);
		}
		if let Some(parent) = current.parent_url() {
			self.parent = Some(self.history_new(&parent));
		}

		// Current
		let rep = self.history_new(&current);
		let rep = mem::replace(&mut self.current, rep);
		if rep.cwd.is_regular() {
			self.history.insert(rep.cwd.clone(), rep);
		}

		// Backstack
		self.backstack.push(current);

		Manager::_refresh();
		render!();
	}
}
