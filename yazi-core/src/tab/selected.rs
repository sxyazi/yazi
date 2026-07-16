use hashbrown::HashMap;
use indexmap::{IndexMap, map::MutableKeys};
use yazi_fs::{FilesOp, file::{File, FileCov}};
use yazi_shared::{timestamp_us, url::{AsUrl, Url, UrlBuf, UrlBufCov, UrlCov, UrlCovMapExt, UrlLike, UrlMapExt}};

#[derive(Default)]
pub struct Selected {
	inner:   IndexMap<FileCov, u64>,
	parents: HashMap<UrlBufCov, usize>,
}

impl Selected {
	pub fn len(&self) -> usize { self.inner.len() }

	pub fn is_empty(&self) -> bool { self.inner.is_empty() }

	pub fn files(&self) -> impl Iterator<Item = &File> { self.inner.keys().map(|f| &f.0) }

	pub fn urls(&self) -> impl Iterator<Item = &UrlBuf> { self.inner.keys().map(|f| &f.url) }

	pub fn contains(&self, url: impl AsUrl) -> bool {
		self.inner.contains_key(&UrlCov::new(url.as_url()))
	}

	pub fn add(&mut self, file: impl AsUrl + Into<File>) -> bool { self.add_same([file]) == 1 }

	pub fn add_many<I, T>(&mut self, files: I) -> usize
	where
		I: IntoIterator<Item = T>,
		T: AsUrl + Into<File>,
	{
		let mut grouped: IndexMap<_, Vec<_>> = Default::default();
		for file in files {
			if let Some(p) = file.as_url().parent() {
				grouped.get_or_insert_default(p).push(file);
			}
		}
		grouped.into_values().map(|v| self.add_same(v)).sum()
	}

	fn add_same<I, T>(&mut self, files: I) -> usize
	where
		I: IntoIterator<Item = T>,
		T: AsUrl + Into<File>,
	{
		// If it has appeared as a parent
		let files: Vec<_> =
			files.into_iter().filter(|f| !self.parents.contains_key(&UrlCov::new(f.as_url()))).collect();
		if files.is_empty() {
			return 0;
		}

		// If it has appeared as a child
		let mut parent = files[0].as_url().parent();
		while let Some(u) = parent {
			if self.inner.contains_key(&UrlCov::new(u)) {
				return 0;
			}
			parent = u.parent();
		}

		// Mark the files as selected children with a timestamp
		let (now, fl, il) = (timestamp_us(), files.len(), self.inner.len());
		self
			.inner
			.extend(files.into_iter().enumerate().map(|(i, f)| (FileCov(f.into()), now + i as u64)));

		// Update the parent counts
		if let Some((first, _)) = self.inner.get_index(il) {
			let mut parent = first.url.parent();
			while let Some(u) = parent {
				*self.parents.get_or_insert_default(UrlCov::new(u)) += self.inner.len() - il;
				parent = u.parent();
			}
		}

		fl
	}

	pub fn remove(&mut self, url: impl AsUrl) -> bool { self.remove_same([url]) == 1 }

	pub fn remove_many<'a, I, T>(&mut self, urls: I) -> usize
	where
		I: IntoIterator<Item = T>,
		T: Into<Url<'a>>,
	{
		let mut grouped: HashMap<_, Vec<_>> = Default::default();
		for url in urls.into_iter().map(Into::into) {
			if let Some(p) = url.parent() {
				grouped.entry(p).or_default().push(url);
			}
		}

		let affected = grouped.into_values().map(|v| self.remove_same(v)).sum();
		if affected > 0 {
			self.inner.sort_unstable_by(|_, a, _, b| a.cmp(b));
		}

		affected
	}

	fn remove_same<I, T>(&mut self, urls: I) -> usize
	where
		I: IntoIterator<Item = T>,
		T: AsUrl,
	{
		let mut it = urls.into_iter();
		let Some(first) = it.next() else { return 0 };

		let count = self.inner.swap_remove(&UrlCov::new(first.as_url())).is_some() as usize
			+ it.filter_map(|u| self.inner.swap_remove(&UrlCov::new(u.as_url()))).count();
		if count == 0 {
			return 0;
		}

		let mut parent = first.as_url().parent();
		while let Some(u) = parent {
			let n = self.parents.get_mut(&UrlCov::new(u)).unwrap();

			*n -= count;
			if *n == 0 {
				self.parents.remove(&UrlCov::new(u));
			}

			parent = u.parent();
		}
		count
	}

	pub fn clear(&mut self) {
		self.inner.clear();
		self.parents.clear();
	}

	pub fn apply_op(&mut self, op: &FilesOp) {
		let (removal, addition) = op.diff_recoverable(self.urls());
		if !removal.is_empty() {
			self.remove_many(&removal);
		}
		if !addition.is_empty() {
			self.add_many(addition);
		}
		for f in op.files() {
			self.inner.get_full_mut2(&UrlCov::new(&f.url)).map(|(_, k, _)| *k = f.into());
		}
	}
}

#[cfg(test)]
mod tests {
	use std::{ffi::OsStr, path::Path};

	use super::*;

	fn f<S: AsRef<OsStr> + ?Sized>(s: &S) -> File { File::from_dummy(Path::new(s), None) }

	#[test]
	fn test_insert_non_conflicting() {
		let mut s = Selected::default();

		assert!(s.add(f("/a/b")));
		assert!(s.add(f("/c/d")));
		assert_eq!(s.inner.len(), 2);
	}

	#[test]
	fn test_insert_conflicting_parent() {
		let mut s = Selected::default();

		assert!(s.add(f("/a")));
		assert!(!s.add(f("/a/b")));
	}

	#[test]
	fn test_insert_conflicting_child() {
		let mut s = Selected::default();

		assert!(s.add(f("/a/b/c")));
		assert!(!s.add(f("/a/b")));
		assert!(s.add(f("/a/b/d")));
	}

	#[test]
	fn test_remove() {
		let mut s = Selected::default();

		assert!(s.add(f("/a/b")));
		assert!(!s.remove(Path::new("/a/c")));
		assert!(s.remove(Path::new("/a/b")));
		assert!(!s.remove(Path::new("/a/b")));
		assert!(s.inner.is_empty());
		assert!(s.parents.is_empty());

		// Relative path with an empty parent (root)
		assert!(s.add(f("a")));
		assert!(s.remove(Path::new("a")));
		assert!(s.inner.is_empty());
		assert!(s.parents.is_empty());
	}

	#[test]
	fn add_many_success() {
		let mut s = Selected::default();

		assert_eq!(3, s.add_same([f("/parent/child1"), f("/parent/child2"), f("/parent/child3")]));
	}

	#[test]
	fn add_many_with_existing_parent_fails() {
		let mut s = Selected::default();

		s.add(f("/parent"));
		assert_eq!(0, s.add_same([f("/parent/child1"), f("/parent/child2")]));
	}

	#[test]
	fn add_many_with_existing_child_fails() {
		let mut s = Selected::default();

		s.add(f("/parent/child1"));
		assert_eq!(2, s.add_same([f("/parent/child1"), f("/parent/child2")]));
	}

	#[test]
	fn add_many_empty_files_list() {
		let mut s = Selected::default();

		assert_eq!(0, s.add_same([] as [File; 0]));
	}

	#[test]
	fn add_many_with_parent_as_child_of_another_url() {
		let mut s = Selected::default();

		s.add(f("/parent/child"));
		assert_eq!(0, s.add_same([f("/parent/child/child1"), f("/parent/child/child2")]));
	}

	#[test]
	fn add_many_with_direct_parent_fails() {
		let mut s = Selected::default();

		s.add(f("/a"));
		assert_eq!(0, s.add_same([f("/a/b")]));
	}

	#[test]
	fn add_many_with_nested_child_fails() {
		let mut s = Selected::default();

		s.add(f("/a/b"));
		assert_eq!(0, s.add_same([f("/a")]));
		assert_eq!(1, s.add_same([f("/b"), f("/a")]));
	}

	#[test]
	fn add_many_sibling_directories_success() {
		let mut s = Selected::default();

		assert_eq!(2, s.add_same([f("/a/b"), f("/a/c")]));
	}

	#[test]
	fn add_many_with_grandchild_fails() {
		let mut s = Selected::default();

		s.add(f("/a/b"));
		assert_eq!(0, s.add_same([f("/a/b/c")]));
	}

	#[test]
	fn test_add_many_with_remove() {
		let mut s = Selected::default();

		let child1 = Path::new("/parent/child1");
		let child2 = Path::new("/parent/child2");
		let child3 = Path::new("/parent/child3");
		assert_eq!(3, s.add_same([f(child1), f(child2), f(child3)]));

		assert!(s.remove(child1));
		assert_eq!(s.inner.len(), 2);
		assert!(!s.parents.is_empty());

		assert!(s.remove(child2));
		assert!(!s.parents.is_empty());

		assert!(s.remove(child3));
		assert!(s.inner.is_empty());
		assert!(s.parents.is_empty());
	}

	#[test]
	fn add_same_all_duplicates() {
		let mut s = Selected::default();

		s.add(f("/a/b"));
		assert_eq!(1, s.add_same([f("/a/b")]));
		assert_eq!(s.inner.len(), 1);
	}
}
