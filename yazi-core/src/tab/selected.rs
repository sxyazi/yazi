use std::{collections::{BTreeSet, HashMap}, ops::Deref};

use yazi_shared::fs::Url;

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
	pub fn add(&mut self, url: &Url) -> bool { self.add_many(&[url]) }

	pub fn add_many(&mut self, urls: &[&Url]) -> bool {
		if urls.is_empty() {
			return true;
		}

		let mut parent = urls[0].parent_url();
		while let Some(u) = parent {
			if self.inner.contains(&u) {
				return false;
			}
			parent = u.parent_url();
		}

		if self.parents.contains_key(urls[0]) {
			return false;
		}

		let mut parent = urls[0].parent_url();
		while let Some(u) = parent {
			parent = u.parent_url();
			*self.parents.entry(u).or_insert(0) += urls.len();
		}

		self.inner.extend(urls.iter().map(|&u| u.clone()));
		true
	}

	pub fn remove(&mut self, url: &Url) -> bool {
		if !self.inner.remove(url) {
			return false;
		}

		let mut parent = url.parent_url();
		while let Some(u) = parent {
			let n = self.parents.get_mut(&u).unwrap();
			if *n == 1 {
				self.parents.remove(&u);
			} else {
				*n -= 1;
			}

			parent = u.parent_url();
		}
		true
	}

	pub fn clear(&mut self) {
		self.inner.clear();
		self.parents.clear();
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

		assert!(s.add_many(&[
			&Url::from("/parent/child1"),
			&Url::from("/parent/child2"),
			&Url::from("/parent/child3")
		]));
	}

	#[test]
	fn insert_many_with_existing_parent_fails() {
		let mut s = Selected::default();

		s.add(&Url::from("/parent"));
		assert!(!s.add_many(&[&Url::from("/parent/child1"), &Url::from("/parent/child2"),]));
	}

	#[test]
	fn insert_many_with_existing_child_fails() {
		let mut s = Selected::default();

		s.add(&Url::from("/parent/child1"));
		assert!(s.add_many(&[&Url::from("/parent/child1"), &Url::from("/parent/child2")]));
	}

	#[test]
	fn insert_many_empty_urls_list() {
		let mut s = Selected::default();

		assert!(s.add_many(&[]));
	}

	#[test]
	fn insert_many_with_parent_as_child_of_another_url() {
		let mut s = Selected::default();

		s.add(&Url::from("/parent/child"));
		assert!(!s.add_many(&[&Url::from("/parent/child/child1"), &Url::from("/parent/child/child2")]));
	}
	#[test]
	fn insert_many_with_direct_parent_fails() {
		let mut s = Selected::default();

		s.add(&Url::from("/a"));
		assert!(!s.add_many(&[&Url::from("/a/b")]));
	}

	#[test]
	fn insert_many_with_nested_child_fails() {
		let mut s = Selected::default();

		s.add(&Url::from("/a/b"));
		assert!(!s.add_many(&[&Url::from("/a")]));
	}

	#[test]
	fn insert_many_sibling_directories_success() {
		let mut s = Selected::default();

		assert!(s.add_many(&[&Url::from("/a/b"), &Url::from("/a/c")]));
	}

	#[test]
	fn insert_many_with_grandchild_fails() {
		let mut s = Selected::default();

		s.add(&Url::from("/a/b"));
		assert!(!s.add_many(&[&Url::from("/a/b/c")]));
	}

	#[test]
	fn test_insert_many_with_remove() {
		let mut s = Selected::default();

		let child1 = Url::from("/parent/child1");
		let child2 = Url::from("/parent/child2");
		let child3 = Url::from("/parent/child3");
		assert!(s.add_many(&[&child1, &child2, &child3]));

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
