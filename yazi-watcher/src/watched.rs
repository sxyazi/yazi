use std::{ops::Deref, path::{Path, PathBuf}};

use hashbrown::{HashMap, HashSet};
use indexmap::IndexSet;
use yazi_fs::FsUrl;
use yazi_shared::{path::PathDyn, url::{AsUrl, UrlBuf}};

use crate::Watchee;

#[derive(Debug, Default)]
pub struct Watched {
	urls:   HashSet<Watchee<'static>>,
	caches: HashMap<PathBuf, UrlBuf>,
}

impl Deref for Watched {
	type Target = HashSet<Watchee<'static>>;

	fn deref(&self) -> &Self::Target { &self.urls }
}

impl Watched {
	pub(super) fn insert(&mut self, watchee: Watchee<'static>) {
		let url = watchee.as_url();
		if let Some(cache) = url.cache_bucket() {
			self.caches.insert(cache, url.into());
		}
		self.urls.insert(watchee);
	}

	pub(super) fn remove(&mut self, watchee: &Watchee<'static>) -> bool {
		if !self.urls.remove(watchee) {
			return false;
		}
		if let Some(cache) = watchee.as_url().cache_bucket() {
			self.caches.remove(&cache);
		}
		true
	}

	pub(super) fn difference<'a>(
		&'a self,
		other: &'a IndexSet<Watchee<'static>>,
	) -> impl Iterator<Item = &'a Watchee<'static>> {
		self.urls.iter().filter(move |&watchee| !other.contains(watchee))
	}

	pub(super) fn contains_url(&self, url: impl AsUrl) -> bool {
		let url = url.as_url();
		if url.as_local().is_some() {
			self.urls.contains(&Watchee::Local(url.into(), false))
				|| self.urls.contains(&Watchee::Local(url.into(), true))
		} else {
			self.urls.contains(&Watchee::Virtual(url.into()))
		}
	}

	pub(super) fn contains_path(&self, path: &Path) -> bool {
		self.urls.iter().any(|watchee| watchee.as_url().as_local() == Some(path))
	}

	pub(super) fn paths(&self) -> impl Iterator<Item = &Path> {
		self.urls.iter().filter_map(|watchee| watchee.as_url().as_local())
	}

	pub(super) fn find_by_cache(&self, cache: PathDyn) -> Option<UrlBuf> {
		self.caches.get(cache.as_os().ok()?).cloned()
	}
}
