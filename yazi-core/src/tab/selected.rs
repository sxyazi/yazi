use std::{collections::{BTreeSet, HashMap}, path::PathBuf};

use yazi_shared::fs::Url;

#[derive(Default)]
pub struct Selected {
	inner:   BTreeSet<Url>,
	parents: HashMap<PathBuf, usize>,
}

impl Selected {
	pub fn new() -> Self { Selected { inner: BTreeSet::new(), parents: HashMap::new() } }

	pub fn insert(&mut self, url: Url) -> bool {
		let url_buf = url.to_path_buf();

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
		loop {
			current_path = match current_path.parent() {
				Some(parent) => parent.to_path_buf(),
				None => break,
			};
			let counter = self.parents.entry(current_path.clone()).or_insert(0);
			*counter += 1;
		}
		self.inner.insert(url.clone());
		true
	}

	pub fn remove(&mut self, url: &Url) -> bool {
		if !self.inner.remove(&url) {
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
		return true;
	}

	pub fn is_empty(&self) -> bool { self.inner.is_empty() }

	pub fn clear(&mut self) {
		self.parents.clear();
		self.inner.clear();
	}

	pub fn iter(&self) -> std::collections::btree_set::Iter<Url> { self.inner.iter() }
}
#[cfg(test)]
mod tests {
	use std::path::Path;

	use super::*;

	#[test]
	fn test_insert_non_conflicting() {
		let mut selected = Selected::new();
		let url1 = Url::from(Path::new("/a/b"));
		let url2 = Url::from(Path::new("/c/d"));

		assert!(selected.insert(url1), "Should successfully insert url1");
		assert!(selected.insert(url2), "Should successfully insert url2");
		assert_eq!(selected.inner.len(), 2, "There should be two URLs");
	}

	#[test]
	fn test_insert_conflicting_parent() {
		let mut selected = Selected::new();
		let parent_url = Url::from(Path::new("/a"));
		let child_url = Url::from(Path::new("/a/b"));

		assert!(selected.insert(parent_url), "Should successfully insert parent_url");
		assert!(!selected.insert(child_url), "Should fail to insert child_url due to conflict");
	}

	#[test]
	fn test_insert_conflicting_child() {
		let mut selected = Selected::new();
		let child_url = Url::from(Path::new("/a/b/c"));
		let parent_url = Url::from(Path::new("/a/b"));
		let sibling_url = Url::from(Path::new("/a/b/d"));

		assert!(selected.insert(child_url), "Should successfully insert child_url");
		assert!(!selected.insert(parent_url), "Should fail to insert parent_url due to conflict");
		assert!(selected.insert(sibling_url), "Should successfully insert sibling_url");
	}

	#[test]
	fn test_remove() {
		let mut selected = Selected::new();
		let url = Url::from(Path::new("/a/b"));

		assert!(selected.insert(url.clone()), "Should successfully insert url");
		assert!(selected.remove(&url), "Should successfully remove url");
		assert!(selected.inner.is_empty(), "Inner set should be empty after removal");
		assert!(selected.parents.is_empty(), "Parents map should be empty after removal");
	}
}
