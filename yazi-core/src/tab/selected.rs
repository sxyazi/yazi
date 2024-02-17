use std::{collections::{BTreeSet, HashMap}, path::PathBuf};

use yazi_shared::fs::Url;

#[derive(Default)]
pub struct Selected {
	inner:   BTreeSet<Url>,
	parents: HashMap<PathBuf, usize>,
}

impl Selected {
	pub fn get_inner(&self)->BTreeSet<Url>{
		return self.inner.clone()
	}
	pub fn new() -> Self { Selected { inner: BTreeSet::new(), parents: HashMap::new() } }

	pub fn insert(&mut self, url: Url) -> bool { self.insert_many(&[&url]) }

	pub fn insert_many(&mut self, urls: &[&Url]) -> bool {
		if urls.is_empty() {
			return false;
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
	pub fn contains(&self,url:&Url)->bool{
		self.inner.contains(url)
	}
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

	#[test]
	fn insert_many_success() {
		let mut selected = Selected::new();
		let child1 = Url::from(Path::new("/parent/child1"));
		let child2 = Url::from(Path::new("/parent/child2"));
		let child3 = Url::from(Path::new("/parent/child3"));
		let urls = vec![&child1, &child2, &child3];
		assert!(selected.insert_many(&urls), "Should successfully insert urls with the same parent");
	}

	#[test]
	fn insert_many_with_existing_parent_fails() {
		let mut selected = Selected::new();
		selected.insert(Url::from(Path::new("/parent")));

		let childs1 = Url::from(Path::new("/parent/child1"));
		let childs2 = Url::from(Path::new("/parent/child2"));
		let urls = vec![&childs1, &childs2];
		assert!(!selected.insert_many(&urls), "Should fail to insert since parent already exists");
	}

	#[test]
	fn insert_many_with_existing_child_fails() {
		let mut selected = Selected::new();
		let child = Url::from(Path::new("/parent/child1"));
		selected.insert(child);

		let child1 = Url::from(Path::new("/parent/child1"));
		let child2 = Url::from(Path::new("/parent/child2"));
		let urls = vec![&child1, &child2];
		assert!(
			selected.insert_many(&urls),
			"Should success to insert since one of the children already exists"
		);
	}

	#[test]
	fn insert_many_empty_urls_list() {
		let mut selected = Selected::new();
		let urls = vec![];
		assert!(!selected.insert_many(&urls), "Inserting an empty list of urls should false");
	}

	#[test]
	fn insert_many_with_parent_as_child_of_another_url() {
		let mut selected = Selected::new();
		selected.insert(Url::from(Path::new("/parent/child")));
		let child1 = Url::from(Path::new("/parent/child/child1"));
		let child2 = Url::from(Path::new("/parent/child/child2"));
		let urls = vec![&child1, &child2];
		assert!(
			!selected.insert_many(&urls),
			"Should successfully insert urls when parent is a child of another url in the set"
		);
	}
	#[test]
	fn insert_many_with_direct_parent_fails() {
		let mut selected = Selected::new();
		selected.insert(Url::from(Path::new("/a")));
		let binding = Url::from(Path::new("/a/b"));
		let urls = vec![&binding];
		assert!(
			!selected.insert_many(&urls),
			"Should not allow insert when parent is already selected"
		);
	}

	#[test]
	fn insert_many_with_nested_child_fails() {
		let mut selected = Selected::new();
		selected.insert(Url::from(Path::new("/a/b")));
		let binding = Url::from(Path::new("/a"));
		let urls = vec![&binding];
		assert!(
			!selected.insert_many(&urls),
			"Should not allow insert of a parent when a child is already selected"
		);
	}

	#[test]
	fn insert_many_sibling_directories_success() {
		let mut selected = Selected::new();
		let child1 = Url::from(Path::new("/a/b"));
		let child2 = Url::from(Path::new("/a/c"));
		let urls = vec![&child1, &child2];
		assert!(selected.insert_many(&urls), "Should allow inserts of sibling directories");
	}

	#[test]
	fn insert_many_with_grandchild_fails() {
		let mut selected = Selected::new();
		selected.insert(Url::from(Path::new("/a/b")));
		let binding = Url::from(Path::new("/a/b/c"));
		let urls = vec![&binding];
		assert!(
			!selected.insert_many(&urls),
			"Should not allow insert of a grandchild when the child is already selected"
		);
	}
	#[test]
	fn test_insert_many_with_remove() {
		let mut selected = Selected::new();
		let child1 = Url::from(Path::new("/parent/child1"));
		let child2 = Url::from(Path::new("/parent/child2"));
		let child3 = Url::from(Path::new("/parent/child3"));
		let urls = vec![&child1, &child2, &child3];
		assert!(selected.insert_many(&urls), "Should successfully insert urls with the same parent");
		assert!(selected.remove(&child1), "Should successfully remove url");
		assert_eq!(selected.inner.len(),2);
		assert!(!selected.parents.is_empty(),"parent map should not be empty");
		assert!(selected.remove(&child2), "Should successfully remove url");
		assert!(!selected.parents.is_empty(),"parent map should not be empty");
		assert!(selected.remove(&child3), "Should successfully remove url");

		assert!(selected.inner.is_empty(), "Inner set should be empty after removal");
		assert!(selected.parents.is_empty(), "Parents map should be empty after removal");
	}
}
