use std::{collections::BTreeSet, env};

use crate::{emit, manager::Manager};

impl Manager {
	pub fn refresh(&mut self) {
		env::set_current_dir(self.cwd()).ok();

		self.active_mut().apply_files_attrs(false);

		if let Some(f) = self.parent() {
			self.watcher.trigger_dirs(&[self.cwd(), &f.cwd]);
		} else {
			self.watcher.trigger_dirs(&[self.cwd()]);
		}
		emit!(Hover);

		let mut to_watch = BTreeSet::new();
		for tab in self.tabs.iter() {
			to_watch.insert(&tab.current.cwd);
			match tab.current.hovered() {
				Some(h) if h.is_dir() => _ = to_watch.insert(&h.url),
				_ => {}
			}
			if let Some(ref p) = tab.parent {
				to_watch.insert(&p.cwd);
			}
		}
		self.watcher.watch(to_watch);
	}
}
