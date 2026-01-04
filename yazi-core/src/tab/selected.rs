use std::ops::Deref;

use hashbrown::HashMap;
use indexmap::IndexMap;
use yazi_fs::FilesOp;
use yazi_shared::{timestamp_us, url::{Url, UrlBuf, UrlBufCov, UrlCov}};

#[derive(Default)]
pub struct Selected {
	inner:   IndexMap<UrlBufCov, u64>,
	parents: HashMap<UrlBufCov, usize>,
}

impl Selected {
	pub fn len(&self) -> usize { self.inner.len() }

	pub fn is_empty(&self) -> bool { self.inner.is_empty() }

	pub fn values(&self) -> impl Iterator<Item = &UrlBuf> { self.inner.keys().map(Deref::deref) }

	pub fn contains<'a>(&self, url: impl Into<Url<'a>>) -> bool {
		self.inner.contains_key(&UrlCov::new(url))
	}

	pub fn add<'a>(&mut self, url: impl Into<Url<'a>>) -> bool { self.add_same([url]) == 1 }

	pub fn add_many<'a, I, T>(&mut self, urls: I) -> usize
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
		grouped.into_values().map(|v| self.add_same(v)).sum()
	}

	fn add_same<'a, I, T>(&mut self, urls: I) -> usize
	where
		I: IntoIterator<Item = T>,
		T: Into<Url<'a>>,
	{
		// If it has appeared as a parent
		let urls: Vec<_> =
			urls.into_iter().map(UrlCov::new).filter(|u| !self.parents.contains_key(u)).collect();
		if urls.is_empty() {
			return 0;
		}

		// If it has appeared as a child
		let mut parent = urls[0].parent();
		let mut parents = vec![];
		while let Some(u) = parent {
			if self.inner.contains_key(&UrlCov::new(u)) {
				return 0;
			}

			parent = u.parent();
			parents.push(u);
		}

		let (now, len) = (timestamp_us(), self.inner.len());
		self.inner.extend(urls.iter().enumerate().map(|(i, u)| (u.into(), now + i as u64)));

		for u in parents {
			*self.parents.entry_ref(&UrlCov::new(u)).or_default() += self.inner.len() - len;
		}
		urls.len()
	}

	#[inline]
	pub fn remove<'a>(&mut self, url: impl Into<Url<'a>> + Clone) -> bool {
		self.remove_same([url]) == 1
	}

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

	fn remove_same<'a, I, T>(&mut self, urls: I) -> usize
	where
		I: IntoIterator<Item = T>,
		T: Into<Url<'a>> + Clone,
	{
		let mut it = urls.into_iter().peekable();
		let Some(first) = it.peek().cloned().map(UrlCov::new) else { return 0 };

		let count = it.filter_map(|u| self.inner.swap_remove(&UrlCov::new(u))).count();
		if count == 0 {
			return 0;
		}

		// FIXME: use UrlCov::parent() instead
		let mut parent = first.parent();
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
		let (removal, addition) = op.diff_recoverable(|u| self.contains(u));
		if !removal.is_empty() {
			self.remove_many(&removal);
		}
		if !addition.is_empty() {
			self.add_many(&addition);
		}
	}
}

#[cfg(test)]
mod tests {
	use std::path::Path;

	use super::*;

	#[test]
	fn test_insert_non_conflicting() {
		let mut s = Selected::default();

		assert!(s.add(Path::new("/a/b")));
		assert!(s.add(Path::new("/c/d")));
		assert_eq!(s.inner.len(), 2);
	}

	#[test]
	fn test_insert_conflicting_parent() {
		let mut s = Selected::default();

		assert!(s.add(Path::new("/a")));
		assert!(!s.add(Path::new("/a/b")));
	}

	#[test]
	fn test_insert_conflicting_child() {
		let mut s = Selected::default();

		assert!(s.add(Path::new("/a/b/c")));
		assert!(!s.add(Path::new("/a/b")));
		assert!(s.add(Path::new("/a/b/d")));
	}

	#[test]
	fn test_remove() {
		let mut s = Selected::default();

		assert!(s.add(Path::new("/a/b")));
		assert!(!s.remove(Path::new("/a/c")));
		assert!(s.remove(Path::new("/a/b")));
		assert!(!s.remove(Path::new("/a/b")));
		assert!(s.inner.is_empty());
		assert!(s.parents.is_empty());
	}

	#[test]
	fn insert_many_success() {
		let mut s = Selected::default();

		assert_eq!(
			3,
			s.add_same([
				Path::new("/parent/child1"),
				Path::new("/parent/child2"),
				Path::new("/parent/child3")
			])
		);
	}

	#[test]
	fn insert_many_with_existing_parent_fails() {
		let mut s = Selected::default();

		s.add(Path::new("/parent"));
		assert_eq!(0, s.add_same([Path::new("/parent/child1"), Path::new("/parent/child2")]));
	}

	#[test]
	fn insert_many_with_existing_child_fails() {
		let mut s = Selected::default();

		s.add(Path::new("/parent/child1"));
		assert_eq!(2, s.add_same([Path::new("/parent/child1"), Path::new("/parent/child2")]));
	}

	#[test]
	fn insert_many_empty_urls_list() {
		let mut s = Selected::default();

		assert_eq!(0, s.add_same([] as [Url; 0]));
	}

	#[test]
	fn insert_many_with_parent_as_child_of_another_url() {
		let mut s = Selected::default();

		s.add(Path::new("/parent/child"));
		assert_eq!(
			0,
			s.add_same([Path::new("/parent/child/child1"), Path::new("/parent/child/child2")])
		);
	}
	#[test]
	fn insert_many_with_direct_parent_fails() {
		let mut s = Selected::default();

		s.add(Path::new("/a"));
		assert_eq!(0, s.add_same([Path::new("/a/b")]));
	}

	#[test]
	fn insert_many_with_nested_child_fails() {
		let mut s = Selected::default();

		s.add(Path::new("/a/b"));
		assert_eq!(0, s.add_same([Path::new("/a")]));
		assert_eq!(1, s.add_same([Path::new("/b"), Path::new("/a")]));
	}

	#[test]
	fn insert_many_sibling_directories_success() {
		let mut s = Selected::default();

		assert_eq!(2, s.add_same([Path::new("/a/b"), Path::new("/a/c")]));
	}

	#[test]
	fn insert_many_with_grandchild_fails() {
		let mut s = Selected::default();

		s.add(Path::new("/a/b"));
		assert_eq!(0, s.add_same([Path::new("/a/b/c")]));
	}

	#[test]
	fn test_insert_many_with_remove() {
		let mut s = Selected::default();

		let child1 = Path::new("/parent/child1");
		let child2 = Path::new("/parent/child2");
		let child3 = Path::new("/parent/child3");
		assert_eq!(3, s.add_same([child1, child2, child3]));

		assert!(s.remove(child1));
		assert_eq!(s.inner.len(), 2);
		assert!(!s.parents.is_empty());

		assert!(s.remove(child2));
		assert!(!s.parents.is_empty());

		assert!(s.remove(child3));
		assert!(s.inner.is_empty());
		assert!(s.parents.is_empty());
	}
}
