use yazi_shared::{event::Cmd, render};

use crate::manager::Manager;

impl Manager {
	pub fn unyank(&mut self, _: Cmd) {
		self.yanked = Default::default();
		render!();
	}
}
