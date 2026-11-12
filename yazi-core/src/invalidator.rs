use hashbrown::HashSet;
use yazi_fs::op::FilesOp;
use yazi_shared::{path::PathBufDyn, url::{AsUrl, Url}};

use crate::{mgr::Mgr, tab::Tab};

pub struct Invalidator<'a> {
	tabs: &'a mut [Tab],
}

impl<'a> Invalidator<'a> {
	pub fn new(mgr: &'a mut Mgr) -> Self { Self { tabs: &mut mgr.tabs.items } }

	pub fn apply(&mut self, op: &FilesOp) {
		match op {
			FilesOp::Delete(trail, keys) => self.invalidate_keys(trail.as_url(), keys),
			FilesOp::Upsert(_, files) => {
				for file in files.values() {
					self.invalidate(file.as_url());
				}
			}
			_ => {}
		}
	}

	pub fn invalidate(&mut self, url: Url<'_>) {
		for tab in &mut *self.tabs {
			if tab.current.url == url {
				tab.current.invalidate();
			}
			if let Some(parent) = tab.parent.as_mut().filter(|f| f.url == url) {
				parent.invalidate();
			}
			if let Some(folder) = tab.history.get_mut(url) {
				folder.invalidate();
			}
		}
	}

	fn invalidate_keys(&mut self, trail: Url, keys: &HashSet<PathBufDyn>) {
		let matches = |url: Url| url.pair().is_some_and(|(t, k)| t == trail && keys.contains(&k));

		for tab in &mut *self.tabs {
			if matches(tab.current.as_url()) {
				tab.current.invalidate();
			}
			if let Some(parent) = tab.parent.as_mut().filter(|f| matches(f.as_url())) {
				parent.invalidate();
			}

			tab.backstack.remove_keys(trail, keys);
			tab.history.for_each_mut(trail, keys, |folder| folder.invalidate());
		}
	}
}
