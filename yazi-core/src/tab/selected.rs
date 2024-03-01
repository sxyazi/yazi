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
	#[inline]
	pub fn add(&mut self, url: &Url) -> bool { self.add_many(&[url]) == 1 }

	/// Adds a list of URLs to the user structure.
	///
	/// This method attempts to add a slice of `Url` references to the internal
	/// structure, ensuring that all URLs have the same parent directory. For
	/// example, URLs such as `/a/b/c`, `/a/b/d`, `/a/b/e`, and `/a/b/f` are
	/// acceptable, while `/a/b/c` and `/a/e/f` would not be, due to differing
	/// parent directories.
	///
	/// The addition will fail under the following conditions:
	/// - Any of the URLs already exists within the `inner` collection.
	/// - The parent directory of the URLs already exists as a key in the
	///   `parents` map.
	///
	/// When the provided list of URLs is empty, the method will return `true` as
	/// there are no URLs to process, which is considered a successful operation.
	///
	/// # Arguments
	///
	/// * `urls` - A slice of references to `Url` objects that are to be added.
	/// All URLs should have the same parent path.
	///
	/// # Returns
	///
	/// Return the number of URLs that did not conflict,
	/// even if they were already present in the structure and were not added.
	///
	/// # Examples
	///
	/// ```
	/// # use yazi_core::tab::Selected;
	/// # use yazi_shared::fs::Url;
	/// let mut s = Selected::default();
	///
	/// let url1 = Url::from("/a/b/c");
	/// let url2 = Url::from("/a/b/d");
	/// assert!(s.add_many(&[&url1, &url2]));
	/// ```
	pub fn add_many(&mut self, urls: &[&Url]) -> usize {
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
	pub fn remove(&mut self, url: &Url) -> bool { self.remove_many(&[url]) == 1 }

	pub fn remove_many(&mut self, urls: &[&Url]) -> usize {
		let count = urls.iter().map(|&u| self.inner.remove(u)).filter(|&b| b).count();
		if count == 0 {
			return 0;
		}

		let mut parent = urls[0].parent_url();
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
			s.add_many(&[
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
		assert_eq!(0, s.add_many(&[&Url::from("/parent/child1"), &Url::from("/parent/child2")]));
	}

	#[test]
	fn insert_many_with_existing_child_fails() {
		let mut s = Selected::default();

		s.add(&Url::from("/parent/child1"));
		assert_eq!(2, s.add_many(&[&Url::from("/parent/child1"), &Url::from("/parent/child2")]));
	}

	#[test]
	fn insert_many_empty_urls_list() {
		let mut s = Selected::default();

		assert_eq!(0, s.add_many(&[]));
	}

	#[test]
	fn insert_many_with_parent_as_child_of_another_url() {
		let mut s = Selected::default();

		s.add(&Url::from("/parent/child"));
		assert_eq!(
			0,
			s.add_many(&[&Url::from("/parent/child/child1"), &Url::from("/parent/child/child2")])
		);
	}
	#[test]
	fn insert_many_with_direct_parent_fails() {
		let mut s = Selected::default();

		s.add(&Url::from("/a"));
		assert_eq!(0, s.add_many(&[&Url::from("/a/b")]));
	}

	#[test]
	fn insert_many_with_nested_child_fails() {
		let mut s = Selected::default();

		s.add(&Url::from("/a/b"));
		assert_eq!(0, s.add_many(&[&Url::from("/a")]));
		assert_eq!(1, s.add_many(&[&Url::from("/b"), &Url::from("/a")]));
	}

	#[test]
	fn insert_many_sibling_directories_success() {
		let mut s = Selected::default();

		assert_eq!(2, s.add_many(&[&Url::from("/a/b"), &Url::from("/a/c")]));
	}

	#[test]
	fn insert_many_with_grandchild_fails() {
		let mut s = Selected::default();

		s.add(&Url::from("/a/b"));
		assert_eq!(0, s.add_many(&[&Url::from("/a/b/c")]));
	}

	#[test]
	fn test_insert_many_with_remove() {
		let mut s = Selected::default();

		let child1 = Url::from("/parent/child1");
		let child2 = Url::from("/parent/child2");
		let child3 = Url::from("/parent/child3");
		assert_eq!(3, s.add_many(&[&child1, &child2, &child3]));

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
