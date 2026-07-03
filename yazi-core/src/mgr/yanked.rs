use std::ops::Deref;

use hashbrown::HashSet;
use indexmap::{IndexSet, set::MutableValues};
use yazi_dds::Pubsub;
use yazi_fs::{FilesOp, file::FileCov};
use yazi_macro::err;
use yazi_shared::url::{Url, UrlBuf, UrlCov, UrlLike};

#[derive(Debug, Default)]
pub struct Yanked {
	pub cut: bool,
	files:   IndexSet<FileCov>,

	version:  u64,
	revision: u64,
}

impl Deref for Yanked {
	type Target = IndexSet<FileCov>;

	fn deref(&self) -> &Self::Target { &self.files }
}

impl Yanked {
	pub fn new(cut: bool, files: IndexSet<FileCov>) -> Self {
		Self { cut, files, ..Default::default() }
	}

	pub fn urls(&self) -> impl Iterator<Item = &UrlBuf> { self.files.iter().map(|f| &f.url) }

	pub fn remove_many<'a, I, T>(&mut self, urls: I)
	where
		I: IntoIterator<Item = T>,
		T: Into<UrlCov<'a>>,
	{
		let urls: HashSet<_> = urls.into_iter().map(Into::into).collect();
		if urls.is_empty() {
			return;
		}

		let old = self.files.len();
		self.files.retain(|f| !urls.contains(&UrlCov::new(&f.url)));
		self.revision += (old != self.files.len()) as u64;
	}

	pub fn clear(&mut self) {
		if self.files.is_empty() {
			return;
		}

		self.files.clear();
		self.revision += 1;
	}

	pub fn contains<'a>(&self, url: impl Into<Url<'a>>) -> bool {
		self.files.contains(&UrlCov::new(url))
	}

	pub fn contains_in(&self, dir: &UrlBuf) -> bool {
		self.files.iter().any(|f| {
			let mut it = f.url.components();
			it.next_back().is_some()
				&& it.covariant(&dir.components())
				&& f.url.parent().is_some_and(|p| p == *dir)
		})
	}

	pub fn apply_op(&mut self, op: &FilesOp) {
		let (removal, addition) = op.diff_recoverable(|u| self.contains(u));
		if !removal.is_empty() {
			let old = self.files.len();
			self.files.retain(|f| !removal.iter().any(|u| f.url.covariant(u)));
			self.revision += (old != self.files.len()) as u64;
		}
		if !addition.is_empty() {
			let old = self.files.len();
			self.files.extend(addition.into_iter().map(FileCov));
			self.revision += (old != self.files.len()) as u64;
		}
		for f in op.files() {
			self.files.get_full_mut2(&UrlCov::new(&f.url)).map(|(_, v)| *v = f.into());
		}
	}

	pub fn catchup_revision(&mut self, force: bool) -> bool {
		if self.version == self.revision && !force {
			return false;
		}

		self.version = self.revision;
		err!(Pubsub::pub_after_yank(self.cut, &self.files));
		true
	}
}
