use std::collections::HashSet;

use yazi_shared::{event::Cmd, render};

use crate::manager::{Manager, Yanked};

pub struct Opt {
	cut: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { cut: c.named.contains_key("cut") } }
}

impl Manager {
	pub fn yank(&mut self, opt: impl Into<Opt>) {
		if !self.active_mut().try_escape_visual() {
			return;
		}

		let selected: HashSet<_> = self.selected_or_hovered(false).into_iter().cloned().collect();
		if selected.is_empty() {
			return;
		}

		self.yanked = Yanked { cut: opt.into().cut, urls: selected };
		self.active_mut().escape_select();
		render!();
	}
}
