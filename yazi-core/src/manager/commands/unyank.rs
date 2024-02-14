use yazi_shared::{event::Cmd, render};

use crate::manager::Manager;

impl Manager {
	pub fn unyank(&mut self, _: Cmd) {
		render!(!self.yanked.is_empty());

		self.yanked = Default::default();
	}
}
