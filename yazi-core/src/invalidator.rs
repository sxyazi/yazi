use hashbrown::HashSet;
use yazi_fs::FilesOp;
use yazi_shared::{path::PathBufDyn, url::{UrlBuf, UrlLike}};

use crate::{mgr::Mgr, tab::Tab};

pub struct Invalidator<'a> {
	tabs: &'a mut [Tab],
}

impl<'a> Invalidator<'a> {
	pub fn new(mgr: &'a mut Mgr) -> Self { Self { tabs: &mut mgr.tabs.items } }

	pub fn apply(&mut self, op: &FilesOp) {
		match op {
			FilesOp::Deleting(trail, keys) => {
				self.invalidate_keys(trail, keys);
			}
			FilesOp::Upserting(_, files) => {
				for file in files.values() {
					self.invalidate(&file.url);
				}
			}
			_ => {}
		}
	}

	fn invalidate(&mut self, url: &UrlBuf) {
		for tab in &mut *self.tabs {
			if tab.current.url == *url {
				tab.current.invalidate();
			}
			if let Some(parent) = tab.parent.as_mut().filter(|f| f.url == *url) {
				parent.invalidate();
			}
			if let Some(folder) = tab.history.get_mut(url) {
				folder.invalidate();
			}
		}
	}

	fn invalidate_keys(&mut self, trail: &UrlBuf, keys: &HashSet<PathBufDyn>) {
		let matches = |url: &UrlBuf| url.pair().is_some_and(|(t, k)| t == *trail && keys.contains(&k));

		for tab in &mut *self.tabs {
			if matches(&tab.current.url) {
				tab.current.invalidate();
			}
			if let Some(parent) = tab.parent.as_mut().filter(|f| matches(&f.url)) {
				parent.invalidate();
			}

			tab.backstack.remove_keys(trail, keys);
			tab.history.for_each_mut(trail, keys, |folder| folder.invalidate());
		}
	}
}
