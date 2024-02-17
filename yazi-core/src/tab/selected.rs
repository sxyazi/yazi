use std::{collections::{BTreeSet, HashMap}, path::PathBuf};

use yazi_shared::fs::Url;

#[derive(Default)]
pub struct Selected {
	inner:   BTreeSet<Url>,
	parents: HashMap<PathBuf, usize>,
}

impl Selected {
	pub fn get_inner(&self) -> BTreeSet<Url> { self.inner.clone() }

	pub fn insert(&mut self, url: Url) -> bool { self.insert_many(&[&url]) }

	pub fn insert_many(&mut self, urls: &[&Url]) -> bool {
		if urls.is_empty() {
			return true;
		}

		let url_buf = urls[0].to_path_buf();

		let mut current_path = url_buf.clone();
		while let Some(parent) = current_path.parent() {
			if self.inner.contains(&Url::from(parent)) {
				return false;
			}
			current_path = parent.to_path_buf();
		}

		if self.parents.contains_key(&url_buf) {
			return false;
		}

		let mut current_path = url_buf.clone();
		let len_of_urls = urls.len();
		loop {
			current_path = match current_path.parent() {
				Some(parent) => parent.to_path_buf(),
				None => break,
			};
			let counter = self.parents.entry(current_path.clone()).or_insert(0);
			*counter += len_of_urls;
		}

		self.inner.extend(urls.iter().cloned().cloned());
		true
	}

	pub fn remove(&mut self, url: &Url) -> bool {
		if !self.inner.remove(url) {
			return false;
		}

		let mut current_path = url.to_path_buf();
		loop {
			current_path = match current_path.parent() {
				Some(parent) => parent.to_path_buf(),
				None => break,
			};
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
	use std::path::Path;

	use super::*;

	#[test]
	fn test_insert_non_conflicting() {
		let mut selected = Selected::default();
		let url1 = Url::from(Path::new("/a/b"));
		let url2 = Url::from(Path::new("/c/d"));

		assert!(selected.insert(url1));
		assert!(selected.insert(url2));
		assert_eq!(selected.inner.len(), 2);
	}

	#[test]
	fn test_insert_conflicting_parent() {
		let mut selected = Selected::default();
		let parent_url = Url::from(Path::new("/a"));
		let child_url = Url::from(Path::new("/a/b"));

		assert!(selected.insert(parent_url));
		assert!(!selected.insert(child_url));
	}

	#[test]
	fn test_insert_conflicting_child() {
		let mut selected = Selected::default();
		let child_url = Url::from(Path::new("/a/b/c"));
		let parent_url = Url::from(Path::new("/a/b"));
		let sibling_url = Url::from(Path::new("/a/b/d"));

		assert!(selected.insert(child_url));
		assert!(!selected.insert(parent_url));
		assert!(selected.insert(sibling_url));
	}

	#[test]
	fn test_remove() {
		let mut selected = Selected::default();
		let url = Url::from(Path::new("/a/b"));

		assert!(selected.insert(url.clone()));
		assert!(selected.remove(&url));
		assert!(selected.inner.is_empty());
		assert!(selected.parents.is_empty());
	}

	#[test]
	fn insert_many_success() {
		let mut selected = Selected::default();
		let child1 = Url::from(Path::new("/parent/child1"));
		let child2 = Url::from(Path::new("/parent/child2"));
		let child3 = Url::from(Path::new("/parent/child3"));
		let urls = vec![&child1, &child2, &child3];
		assert!(selected.insert_many(&urls));
	}

	#[test]
	fn insert_many_with_existing_parent_fails() {
		let mut selected = Selected::default();
		selected.insert(Url::from(Path::new("/parent")));

		let child1 = Url::from(Path::new("/parent/child1"));
		let child2 = Url::from(Path::new("/parent/child2"));
		let urls = vec![&child1, &child2];
		assert!(!selected.insert_many(&urls));
	}

	#[test]
	fn insert_many_with_existing_child_fails() {
		let mut selected = Selected::default();
		let child = Url::from(Path::new("/parent/child1"));
		selected.insert(child);

		let child1 = Url::from(Path::new("/parent/child1"));
		let child2 = Url::from(Path::new("/parent/child2"));
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
		selected.insert(Url::from(Path::new("/parent/child")));
		let child1 = Url::from(Path::new("/parent/child/child1"));
		let child2 = Url::from(Path::new("/parent/child/child2"));
		let urls = vec![&child1, &child2];
		assert!(!selected.insert_many(&urls));
	}
	#[test]
	fn insert_many_with_direct_parent_fails() {
		let mut selected = Selected::default();
		selected.insert(Url::from(Path::new("/a")));
		let binding = Url::from(Path::new("/a/b"));
		let urls = vec![&binding];
		assert!(!selected.insert_many(&urls));
	}

	#[test]
	fn insert_many_with_nested_child_fails() {
		let mut selected = Selected::default();
		selected.insert(Url::from(Path::new("/a/b")));
		let binding = Url::from(Path::new("/a"));
		let urls = vec![&binding];
		assert!(!selected.insert_many(&urls));
	}

	#[test]
	fn insert_many_sibling_directories_success() {
		let mut selected = Selected::default();
		let child1 = Url::from(Path::new("/a/b"));
		let child2 = Url::from(Path::new("/a/c"));
		let urls = vec![&child1, &child2];
		assert!(selected.insert_many(&urls));
	}

	#[test]
	fn insert_many_with_grandchild_fails() {
		let mut selected = Selected::default();
		selected.insert(Url::from(Path::new("/a/b")));
		let binding = Url::from(Path::new("/a/b/c"));
		let urls = vec![&binding];
		assert!(!selected.insert_many(&urls));
	}

	#[test]
	fn test_insert_many_with_remove() {
		let mut selected = Selected::default();
		let child1 = Url::from(Path::new("/parent/child1"));
		let child2 = Url::from(Path::new("/parent/child2"));
		let child3 = Url::from(Path::new("/parent/child3"));
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
