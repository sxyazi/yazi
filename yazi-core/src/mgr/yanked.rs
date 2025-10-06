use std::ops::Deref;

use hashbrown::HashSet;
use yazi_dds::Pubsub;
use yazi_fs::FilesOp;
use yazi_macro::err;
use yazi_shared::url::{Url, UrlBuf, UrlBufCov, UrlCov, UrlLike};

#[derive(Debug, Default)]
pub struct Yanked {
	pub cut: bool,
	urls:    HashSet<UrlBufCov>,

	version:  u64,
	revision: u64,
}

impl Deref for Yanked {
	type Target = HashSet<UrlBufCov>;

	fn deref(&self) -> &Self::Target { &self.urls }
}

impl Yanked {
	pub fn new(cut: bool, urls: HashSet<UrlBufCov>) -> Self {
		Self { cut, urls, ..Default::default() }
	}

	pub fn remove<'a>(&mut self, url: impl Into<Url<'a>>) {
		if self.urls.remove(&UrlCov::new(url)) {
			self.revision += 1;
		}
	}

	pub fn clear(&mut self) {
		if self.urls.is_empty() {
			return;
		}

		self.urls.clear();
		self.revision += 1;
	}

	pub fn contains<'a>(&self, url: impl Into<Url<'a>>) -> bool {
		self.urls.contains(&UrlCov::new(url))
	}

	pub fn contains_in(&self, dir: &UrlBuf) -> bool {
		self.urls.iter().any(|u| {
			let mut it = u.components();
			it.next_back().is_some()
				&& it.covariant(&dir.components())
				&& u.parent().is_some_and(|p| p == *dir)
		})
	}

	pub fn apply_op(&mut self, op: &FilesOp) {
		let (removal, addition) = op.diff_recoverable(|u| self.contains(u));
		if !removal.is_empty() {
			let old = self.urls.len();
			self.urls.retain(|u| !removal.contains(u));
			self.revision += (old != self.urls.len()) as u64;
		}

		if !addition.is_empty() {
			let old = self.urls.len();
			self.urls.extend(addition.into_iter().map(UrlBufCov));
			self.revision += (old != self.urls.len()) as u64;
		}
	}

	pub fn catchup_revision(&mut self, force: bool) -> bool {
		if self.version == self.revision && !force {
			return false;
		}

		self.version = self.revision;
		err!(Pubsub::pub_after_yank(self.cut, &self.urls));
		true
	}
}
