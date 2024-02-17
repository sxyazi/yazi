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

		let mut current_path = url.clone();
		while let Some(parent) = current_path.parent_url() {
			current_path = parent;

			let counter = self.parents.entry(current_path.clone()).or_insert(0);
			*counter -= 1;
			if *counter == 0 {
				self.parents.remove(&current_path);
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
		let mut selected = Selected::default();
		let url1 = Url::from("/a/b");
		let url2 = Url::from("/c/d");

		assert!(selected.insert(url1));
		assert!(selected.insert(url2));
		assert_eq!(selected.inner.len(), 2);
	}

	#[test]
	fn test_insert_conflicting_parent() {
		let mut selected = Selected::default();
		let parent_url = Url::from("/a");
		let child_url = Url::from("/a/b");

		assert!(selected.insert(parent_url));
		assert!(!selected.insert(child_url));
	}

	#[test]
	fn test_insert_conflicting_child() {
		let mut selected = Selected::default();
		let child_url = Url::from("/a/b/c");
		let parent_url = Url::from("/a/b");
		let sibling_url = Url::from("/a/b/d");

		assert!(selected.insert(child_url));
		assert!(!selected.insert(parent_url));
		assert!(selected.insert(sibling_url));
	}

	#[test]
	fn test_remove() {
		let mut selected = Selected::default();
		let url = Url::from("/a/b");

		assert!(selected.insert(url.clone()));
		assert!(selected.remove(&url));
		assert!(selected.inner.is_empty());
		assert!(selected.parents.is_empty());
	}

	#[test]
	fn insert_many_success() {
		let mut selected = Selected::default();
		let child1 = Url::from("/parent/child1");
		let child2 = Url::from("/parent/child2");
		let child3 = Url::from("/parent/child3");
		let urls = vec![&child1, &child2, &child3];
		assert!(selected.insert_many(&urls));
	}

	#[test]
	fn insert_many_with_existing_parent_fails() {
		let mut selected = Selected::default();
		selected.insert(Url::from("/parent"));

		let child1 = Url::from("/parent/child1");
		let child2 = Url::from("/parent/child2");
		let urls = vec![&child1, &child2];
		assert!(!selected.insert_many(&urls));
	}

	#[test]
	fn insert_many_with_existing_child_fails() {
		let mut selected = Selected::default();
		let child = Url::from("/parent/child1");
		selected.insert(child);

		let child1 = Url::from("/parent/child1");
		let child2 = Url::from("/parent/child2");
		let urls = vec![&child1, &child2];
		assert!(selected.insert_many(&urls));
	}

	#[test]
	fn insert_many_empty_urls_list() {
		let mut selected = Selected::default();
		assert!(selected.insert_many(&[]));
	}

	#[test]
	fn insert_many_with_parent_as_child_of_another_url() {
		let mut selected = Selected::default();
		selected.insert(Url::from("/parent/child"));
		let child1 = Url::from("/parent/child/child1");
		let child2 = Url::from("/parent/child/child2");
		let urls = vec![&child1, &child2];
		assert!(!selected.insert_many(&urls));
	}
	#[test]
	fn insert_many_with_direct_parent_fails() {
		let mut selected = Selected::default();
		selected.insert(Url::from("/a"));
		let binding = Url::from("/a/b");
		let urls = vec![&binding];
		assert!(!selected.insert_many(&urls));
	}

	#[test]
	fn insert_many_with_nested_child_fails() {
		let mut selected = Selected::default();
		selected.insert(Url::from("/a/b"));
		let binding = Url::from("/a");
		let urls = vec![&binding];
		assert!(!selected.insert_many(&urls));
	}

	#[test]
	fn insert_many_sibling_directories_success() {
		let mut selected = Selected::default();
		let child1 = Url::from("/a/b");
		let child2 = Url::from("/a/c");
		let urls = vec![&child1, &child2];
		assert!(selected.insert_many(&urls));
	}

	#[test]
	fn insert_many_with_grandchild_fails() {
		let mut selected = Selected::default();
		selected.insert(Url::from("/a/b"));
		let binding = Url::from("/a/b/c");
		let urls = vec![&binding];
		assert!(!selected.insert_many(&urls));
	}

	#[test]
	fn test_insert_many_with_remove() {
		let mut selected = Selected::default();
		let child1 = Url::from("/parent/child1");
		let child2 = Url::from("/parent/child2");
		let child3 = Url::from("/parent/child3");
		let urls = vec![&child1, &child2, &child3];
		assert!(selected.insert_many(&urls));
		assert!(selected.remove(&child1));
		assert_eq!(selected.inner.len(), 2);
		assert!(!selected.parents.is_empty());
		assert!(selected.remove(&child2));
		assert!(!selected.parents.is_empty());
		assert!(selected.remove(&child3));

		assert!(selected.inner.is_empty());
		assert!(selected.parents.is_empty());
	}
}
