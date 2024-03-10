use std::{collections::{BTreeSet, HashMap}, ops::Deref};

use yazi_shared::fs::{FilesOp, Url};

#[derive(Default)]
pub struct Selected {
	inner:   BTreeSet<Url>,
	parents: HashMap<Url, usize>,
}

impl Deref for Selected {
	type Target = BTreeSet<Url>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Selected {
	#[inline]
	pub fn add(&mut self, url: &Url) -> bool { self.add_same(&[url]) == 1 }

	pub fn add_many(&mut self, urls: &[&Url], same: bool) -> usize {
		if same {
			return self.add_same(urls);
		}

		let mut grouped: HashMap<_, Vec<_>> = Default::default();
		for &u in urls {
			if let Some(p) = u.parent_url() {
				grouped.entry(p).or_default().push(u);
			}
		}
		grouped.into_values().map(|v| self.add_same(&v)).sum()
	}

	fn add_same(&mut self, urls: &[&Url]) -> usize {
		// If it has appeared as a parent
		let urls: Vec<_> = urls.iter().filter(|&&u| !self.parents.contains_key(u)).collect();
		if urls.is_empty() {
			return 0;
		}

		// If it has appeared as a child
		let mut parent = urls[0].parent_url();
		let mut parents = vec![];
		while let Some(u) = parent {
			if self.inner.contains(&u) {
				return 0;
			}

			parent = u.parent_url();
			parents.push(u);
		}

		let len = self.inner.len();
		self.inner.extend(urls.iter().map(|&&u| u.clone()));

		for u in parents {
			*self.parents.entry(u).or_insert(0) += self.inner.len() - len;
		}
		urls.len()
	}

	#[inline]
	pub fn remove(&mut self, url: &Url) -> bool { self.remove_same(&[url]) == 1 }

	pub fn remove_many(&mut self, urls: &[impl AsRef<Url>], same: bool) -> usize {
		if same {
			return self.remove_same(urls);
		}

		let mut grouped: HashMap<_, Vec<_>> = Default::default();
		for u in urls {
			if let Some(p) = u.as_ref().parent_url() {
				grouped.entry(p).or_default().push(u);
			}
		}
		grouped.into_values().map(|v| self.remove_same(&v)).sum()
	}

	fn remove_same(&mut self, urls: &[impl AsRef<Url>]) -> usize {
		let count = urls.iter().map(|u| self.inner.remove(u.as_ref())).filter(|&b| b).count();
		if count == 0 {
			return 0;
		}

		let mut parent = urls[0].as_ref().parent_url();
		while let Some(u) = parent {
			let n = self.parents.get_mut(&u).unwrap();

			*n -= count;
			if *n == 0 {
				self.parents.remove(&u);
			}

			parent = u.parent_url();
		}
		count
	}

	pub fn clear(&mut self) {
		self.inner.clear();
		self.parents.clear();
	}

	pub fn apply_op(&mut self, op: &FilesOp) {
		let (removal, addition) = match op {
			FilesOp::Deleting(_, urls) => (urls.iter().collect(), vec![]),
			FilesOp::Updating(_, urls) | FilesOp::Upserting(_, urls) => {
				urls.iter().filter(|(u, _)| self.contains(u)).map(|(u, f)| (u, &f.url)).unzip()
			}
			_ => (vec![], vec![]),
		};

		if !removal.is_empty() {
			self.remove_many(&removal, !op.url().is_search());
		}
		if !addition.is_empty() {
			self.add_many(&addition, !op.url().is_search());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_insert_non_conflicting() {
		let mut s = Selected::default();

		assert!(s.add(&Url::from("/a/b")));
		assert!(s.add(&Url::from("/c/d")));
		assert_eq!(s.inner.len(), 2);
	}

	#[test]
	fn test_insert_conflicting_parent() {
		let mut s = Selected::default();

		assert!(s.add(&Url::from("/a")));
		assert!(!s.add(&Url::from("/a/b")));
	}

	#[test]
	fn test_insert_conflicting_child() {
		let mut s = Selected::default();

		assert!(s.add(&Url::from("/a/b/c")));
		assert!(!s.add(&Url::from("/a/b")));
		assert!(s.add(&Url::from("/a/b/d")));
	}

	#[test]
	fn test_remove() {
		let mut s = Selected::default();

		assert!(s.add(&Url::from("/a/b")));
		assert!(!s.remove(&Url::from("/a/c")));
		assert!(s.remove(&Url::from("/a/b")));
		assert!(!s.remove(&Url::from("/a/b")));
		assert!(s.inner.is_empty());
		assert!(s.parents.is_empty());
	}

	#[test]
	fn insert_many_success() {
		let mut s = Selected::default();

		assert_eq!(
			3,
			s.add_same(&[
				&Url::from("/parent/child1"),
				&Url::from("/parent/child2"),
				&Url::from("/parent/child3")
			])
		);
	}

	#[test]
	fn insert_many_with_existing_parent_fails() {
		let mut s = Selected::default();

		s.add(&Url::from("/parent"));
		assert_eq!(0, s.add_same(&[&Url::from("/parent/child1"), &Url::from("/parent/child2")]));
	}

	#[test]
	fn insert_many_with_existing_child_fails() {
		let mut s = Selected::default();

		s.add(&Url::from("/parent/child1"));
		assert_eq!(2, s.add_same(&[&Url::from("/parent/child1"), &Url::from("/parent/child2")]));
	}

	#[test]
	fn insert_many_empty_urls_list() {
		let mut s = Selected::default();

		assert_eq!(0, s.add_same(&[]));
	}

	#[test]
	fn insert_many_with_parent_as_child_of_another_url() {
		let mut s = Selected::default();

		s.add(&Url::from("/parent/child"));
		assert_eq!(
			0,
			s.add_same(&[&Url::from("/parent/child/child1"), &Url::from("/parent/child/child2")])
		);
	}
	#[test]
	fn insert_many_with_direct_parent_fails() {
		let mut s = Selected::default();

		s.add(&Url::from("/a"));
		assert_eq!(0, s.add_same(&[&Url::from("/a/b")]));
	}

	#[test]
	fn insert_many_with_nested_child_fails() {
		let mut s = Selected::default();

		s.add(&Url::from("/a/b"));
		assert_eq!(0, s.add_same(&[&Url::from("/a")]));
		assert_eq!(1, s.add_same(&[&Url::from("/b"), &Url::from("/a")]));
	}

	#[test]
	fn insert_many_sibling_directories_success() {
		let mut s = Selected::default();

		assert_eq!(2, s.add_same(&[&Url::from("/a/b"), &Url::from("/a/c")]));
	}

	#[test]
	fn insert_many_with_grandchild_fails() {
		let mut s = Selected::default();

		s.add(&Url::from("/a/b"));
		assert_eq!(0, s.add_same(&[&Url::from("/a/b/c")]));
	}

	#[test]
	fn test_insert_many_with_remove() {
		let mut s = Selected::default();

		let child1 = Url::from("/parent/child1");
		let child2 = Url::from("/parent/child2");
		let child3 = Url::from("/parent/child3");
		assert_eq!(3, s.add_same(&[&child1, &child2, &child3]));

		assert!(s.remove(&child1));
		assert_eq!(s.inner.len(), 2);
		assert!(!s.parents.is_empty());

		assert!(s.remove(&child2));
		assert!(!s.parents.is_empty());

		assert!(s.remove(&child3));
		assert!(s.inner.is_empty());
		assert!(s.parents.is_empty());
	}
}
