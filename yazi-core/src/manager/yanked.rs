use std::{collections::HashSet, ops::Deref};

use yazi_dds::Pubsub;
use yazi_shared::fs::{FilesOp, Url};

#[derive(Default)]
pub struct Yanked {
	pub cut: bool,
	urls:    HashSet<Url>,

	version:  u64,
	revision: u64,
}

impl Deref for Yanked {
	type Target = HashSet<Url>;

	fn deref(&self) -> &Self::Target { &self.urls }
}

impl Yanked {
	pub fn new(cut: bool, urls: HashSet<Url>) -> Self {
		Self { cut, urls, version: 0, ..Default::default() }
	}

	pub fn remove(&mut self, url: &Url) {
		if self.urls.remove(url) {
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

	pub fn apply_op(&mut self, op: &FilesOp) {
		let (removal, addition) = match op {
			FilesOp::Deleting(_, urls) => (urls.iter().collect(), vec![]),
			FilesOp::Updating(_, urls) | FilesOp::Upserting(_, urls) => {
				urls.iter().filter(|(u, _)| self.contains(u)).map(|(u, f)| (u, f.url_owned())).unzip()
			}
			_ => (vec![], vec![]),
		};

		if !removal.is_empty() {
			let old = self.urls.len();
			self.urls.retain(|u| !removal.contains(&u));
			self.revision += (old != self.urls.len()) as u64;
		}

		if !addition.is_empty() {
			let old = self.urls.len();
			self.urls.extend(addition);
			self.revision += (old != self.urls.len()) as u64;
		}
	}

	pub fn catchup_revision(&mut self, force: bool) -> bool {
		if self.version == self.revision && !force {
			return false;
		}

		self.version = self.revision;
		Pubsub::pub_from_yank(self.cut, &self.urls);
		true
	}
}
