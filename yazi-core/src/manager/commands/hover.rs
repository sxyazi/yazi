use std::collections::BTreeSet;

use yazi_shared::{emit, event::Exec, fs::Url, render, Layer};

use crate::manager::Manager;

pub struct Opt {
	url: Option<Url>,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { url: e.args.first().map(Url::from) } }
}
impl From<Option<Url>> for Opt {
	fn from(url: Option<Url>) -> Self { Self { url } }
}

impl Manager {
	#[inline]
	pub fn _hover(url: Option<Url>) {
		emit!(Call(
			Exec::call("hover", url.map_or_else(Vec::new, |u| vec![u.to_string()])).vec(),
			Layer::Manager
		));
	}

	pub fn hover(&mut self, opt: impl Into<Opt>) {
		// Hover on the file
		let opt = opt.into() as Opt;
		render!(self.current_mut().repos(opt.url));

		// Re-peek
		self.peek(false);
		self.active_mut().apply_files_attrs();

		// Refresh watcher
		let mut to_watch = BTreeSet::new();
		for tab in self.tabs.iter() {
			to_watch.insert(&tab.current.cwd);
			if let Some(ref p) = tab.parent {
				to_watch.insert(&p.cwd);
			}
			if let Some(h) = tab.current.hovered().filter(|&h| h.is_dir()) {
				to_watch.insert(&h.url);
			}
		}
		self.watcher.watch(to_watch);
	}
}
