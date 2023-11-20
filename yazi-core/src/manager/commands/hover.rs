use std::collections::BTreeSet;

use yazi_config::keymap::{Exec, KeymapLayer};
use yazi_shared::Url;

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
			KeymapLayer::Manager
		));
	}

	pub fn hover(&mut self, opt: impl Into<Opt>) -> bool {
		// Refresh watcher
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

		// Trigger peek
		emit!(Peek);

		// Hover
		let opt = opt.into() as Opt;
		self.current_mut().repos(opt.url)
	}
}
