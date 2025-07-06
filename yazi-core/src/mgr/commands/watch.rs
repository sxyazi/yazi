use std::collections::HashSet;

use yazi_parser::mgr::WatchOpt;

use crate::mgr::Mgr;

impl Mgr {
	#[yazi_codegen::command]
	pub fn watch(&mut self, _: WatchOpt) {
		let mut to_watch = HashSet::with_capacity(3 * self.tabs.len());
		for tab in self.tabs.iter() {
			to_watch.insert(tab.cwd());
			if let Some(ref p) = tab.parent {
				to_watch.insert(&p.url);
			}
			if let Some(h) = tab.hovered().filter(|&h| h.is_dir()) {
				to_watch.insert(&h.url);
			}
		}
		self.watcher.watch(to_watch);
	}
}
