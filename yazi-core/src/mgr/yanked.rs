use std::{collections::HashSet, ops::Deref};

use yazi_dds::Pubsub;
use yazi_fs::FilesOp;
use yazi_macro::err;
use yazi_shared::url::{CovUrl, Url};

#[derive(Debug, Default)]
pub struct Yanked {
	pub cut: bool,
	urls:    HashSet<CovUrl>,

	version:  u64,
	revision: u64,
}

impl Deref for Yanked {
	type Target = HashSet<CovUrl>;

	fn deref(&self) -> &Self::Target { &self.urls }
}

impl Yanked {
	pub fn new(cut: bool, urls: HashSet<CovUrl>) -> Self { Self { cut, urls, ..Default::default() } }

	pub fn remove(&mut self, url: &Url) {
		if self.urls.remove(CovUrl::new(url)) {
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

	#[inline]
	pub fn contains(&self, url: impl AsRef<Url>) -> bool { self.urls.contains(CovUrl::new(&url)) }

	pub fn contains_in(&self, dir: &Url) -> bool {
		self.urls.iter().any(|u| {
			let mut it = u.components();
			it.next_back().is_some()
				&& it.covariant(&dir.components())
				&& u.parent_url().is_some_and(|p| p == *dir)
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
			self.urls.extend(addition.into_iter().map(CovUrl));
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
