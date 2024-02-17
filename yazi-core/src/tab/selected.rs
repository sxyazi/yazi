use std::collections::{BTreeSet, HashMap};

use yazi_shared::fs::Url;

#[derive(Default)]
pub struct Selected {
	inner:   BTreeSet<Url>,
	parents: HashMap<Url, usize>,
}

impl Selected {
	pub fn get_inner(&self) -> BTreeSet<Url> { self.inner.clone() }

	pub fn insert(&mut self, url: Url) -> bool { self.insert_many(&[&url]) }

	pub fn insert_many(&mut self, urls: &[&Url]) -> bool {
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

		self.inner.extend(urls.iter().cloned().cloned());
		true
	}

	pub fn remove(&mut self, url: &Url) -> bool {
		if !self.inner.remove(url) {
			return false;
		}

		let mut parent = url.parent_url();
		while let Some(u) = parent {
			parent = u.parent_url();

			let counter = self.parents.entry(u.clone()).or_insert(0);
			*counter -= 1;
			if *counter == 0 {
				self.parents.remove(&u);
			}
		}
		true
	}

	pub fn is_empty(&self) -> bool { self.inner.is_empty() }

	pub fn clear(&mut self) {
		self.parents.clear();
		self.inner.clear();
	}

	pub fn iter(&self) -> std::collections::btree_set::Iter<Url> { self.inner.iter() }

	pub fn contains(&self, url: &Url) -> bool { self.inner.contains(url) }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_insert_non_conflicting() {
		let mut s = Selected::default();

		assert!(s.insert(Url::from("/a/b")));
		assert!(s.insert(Url::from("/c/d")));
		assert_eq!(s.inner.len(), 2);
	}

	#[test]
	fn test_insert_conflicting_parent() {
		let mut s = Selected::default();

		assert!(s.insert(Url::from("/a")));
		assert!(!s.insert(Url::from("/a/b")));
	}

	#[test]
	fn test_insert_conflicting_child() {
		let mut s = Selected::default();

		assert!(s.insert(Url::from("/a/b/c")));
		assert!(!s.insert(Url::from("/a/b")));
		assert!(s.insert(Url::from("/a/b/d")));
	}

	#[test]
	fn test_remove() {
		let mut s = Selected::default();

		assert!(s.insert(Url::from("/a/b")));
		assert!(s.remove(&Url::from("/a/b")));
		assert!(s.inner.is_empty());
		assert!(s.parents.is_empty());
	}

	#[test]
	fn insert_many_success() {
		let mut s = Selected::default();

		assert!(s.insert_many(&[
			&Url::from("/parent/child1"),
			&Url::from("/parent/child2"),
			&Url::from("/parent/child3")
		]));
	}

	#[test]
	fn insert_many_with_existing_parent_fails() {
		let mut s = Selected::default();

		s.insert(Url::from("/parent"));
		assert!(!s.insert_many(&[&Url::from("/parent/child1"), &Url::from("/parent/child2"),]));
	}

	#[test]
	fn insert_many_with_existing_child_fails() {
		let mut s = Selected::default();

		s.insert(Url::from("/parent/child1"));
		assert!(s.insert_many(&[&Url::from("/parent/child1"), &Url::from("/parent/child2")]));
	}

	#[test]
	fn insert_many_empty_urls_list() {
		let mut s = Selected::default();

		assert!(s.insert_many(&[]));
	}

	#[test]
	fn insert_many_with_parent_as_child_of_another_url() {
		let mut s = Selected::default();

		s.insert(Url::from("/parent/child"));
		assert!(
			!s.insert_many(&[&Url::from("/parent/child/child1"), &Url::from("/parent/child/child2")])
		);
	}
	#[test]
	fn insert_many_with_direct_parent_fails() {
		let mut s = Selected::default();

		s.insert(Url::from("/a"));
		assert!(!s.insert_many(&[&Url::from("/a/b")]));
	}

	#[test]
	fn insert_many_with_nested_child_fails() {
		let mut s = Selected::default();

		s.insert(Url::from("/a/b"));
		assert!(!s.insert_many(&[&Url::from("/a")]));
	}

	#[test]
	fn insert_many_sibling_directories_success() {
		let mut s = Selected::default();

		assert!(s.insert_many(&[&Url::from("/a/b"), &Url::from("/a/c")]));
	}

	#[test]
	fn insert_many_with_grandchild_fails() {
		let mut s = Selected::default();

		s.insert(Url::from("/a/b"));
		assert!(!s.insert_many(&[&Url::from("/a/b/c")]));
	}

	#[test]
	fn test_insert_many_with_remove() {
		let mut s = Selected::default();

		let child1 = Url::from("/parent/child1");
		let child2 = Url::from("/parent/child2");
		let child3 = Url::from("/parent/child3");
		assert!(s.insert_many(&[&child1, &child2, &child3]));

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
