use std::collections::BTreeSet;

use yazi_config::keymap::Exec;
use yazi_shared::{fs::Url, Layer};

use crate::{emit, manager::Manager};

pub struct Opt {
	url: Option<Url>,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { url: e.args.first().map(Url::from) } }
}

impl Manager {
	#[inline]
	pub fn _hover(url: Option<Url>) {
		emit!(Call(
			Exec::call("hover", url.map_or_else(Vec::new, |u| vec![u.to_string()])).vec(),
			Layer::Manager
		));
	}

	pub fn hover(&mut self, opt: impl Into<Opt>) -> bool {
		// Hover on the file
		let opt = opt.into() as Opt;
		let mut b = self.current_mut().repos(opt.url);

		// Re-peek
		b |= self.peek(0);

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

		b
	}
}
