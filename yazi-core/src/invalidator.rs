use yazi_fs::FilesOp;
use yazi_shared::url::{UrlBuf, UrlLike};

use crate::{mgr::Mgr, tab::Tab};

pub struct Invalidator<'a> {
	tabs: &'a mut [Tab],
}

impl<'a> Invalidator<'a> {
	pub fn new(mgr: &'a mut Mgr) -> Self { Self { tabs: &mut mgr.tabs.items } }

	pub fn apply(&mut self, op: &FilesOp) {
		match op {
			FilesOp::Deleting(parent, keys) => {
				for url in keys.iter().filter_map(|key| parent.try_join(key).ok()) {
					self.invalidate(&url);
				}
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
}
