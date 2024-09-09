use std::collections::HashSet;

use yazi_dds::Pubsub;
use yazi_shared::{event::{Cmd, Data}, fs::Url, render};

use crate::manager::Manager;

pub struct Opt {
	url: Option<Url>,
	tab: Option<usize>,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			url: c.take_first().and_then(Data::into_url),
			tab: c.get("tab").and_then(Data::as_usize),
		}
	}
}
impl From<Option<Url>> for Opt {
	fn from(url: Option<Url>) -> Self { Self { url, tab: None } }
}

impl Manager {
	pub fn hover(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;

		// Hover on the file
		render!(self.current_or_mut(opt.tab).repos(opt.url.as_ref()));
		if opt.url.zip(self.current_or(opt.tab).hovered()).is_some_and(|(u, f)| &u == f.url()) {
			// `hover(Some)` occurs after user actions, such as create, rename, reveal, etc.
			// At this point, it's intuitive to track the location of this file regardless.
			self.current_or_mut(opt.tab).tracing = true;
		}

		// Repeek
		self.peek(false);

		// Refresh watcher
		let mut to_watch = HashSet::with_capacity(3 * self.tabs.len());
		for tab in self.tabs.iter() {
			to_watch.insert(tab.cwd());
			if let Some(ref p) = tab.parent {
				to_watch.insert(&p.loc);
			}
			if let Some(h) = tab.current.hovered().filter(|&h| h.is_dir()) {
				to_watch.insert(h.url());
			}
		}
		self.watcher.watch(to_watch);

		// Publish through DDS
		Pubsub::pub_from_hover(self.active().idx, self.hovered().map(|h| h.url()));
	}
}
