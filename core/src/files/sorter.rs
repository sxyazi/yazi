use std::cmp::Ordering;

use config::{manager::SortBy, MANAGER};

use super::File;

#[derive(PartialEq)]
pub struct FilesSorter {
	pub by:        SortBy,
	pub reverse:   bool,
	pub dir_first: bool,
}

impl Default for FilesSorter {
	fn default() -> Self {
		Self {
			by:        MANAGER.sort_by,
			reverse:   MANAGER.sort_reverse,
			dir_first: MANAGER.sort_dir_first,
		}
	}
}

impl FilesSorter {
	pub(super) fn sort(&self, items: &mut Vec<File>) -> bool {
		if items.is_empty() {
			return false;
		}

		match self.by {
			SortBy::Alphabetical => {
				items.sort_unstable_by(|a, b| self.cmp(&a.path, &b.path, self.promote(a, b)))
			}
			SortBy::Created => items.sort_unstable_by(|a, b| {
				if let (Ok(aa), Ok(bb)) = (a.meta.created(), b.meta.created()) {
					return self.cmp(aa, bb, self.promote(a, b));
				}
				Ordering::Equal
			}),
			SortBy::Modified => items.sort_unstable_by(|a, b| {
				if let (Ok(aa), Ok(bb)) = (a.meta.modified(), b.meta.modified()) {
					return self.cmp(aa, bb, self.promote(a, b));
				}
				Ordering::Equal
			}),
			SortBy::Natural => self.sort_naturally(items),
			SortBy::Size => items.sort_unstable_by(|a, b| {
				self.cmp(a.length.unwrap_or(0), b.length.unwrap_or(0), self.promote(a, b))
			}),
		}
		true
	}

	fn sort_naturally(&self, items: &mut Vec<File>) {
		let mut indices = Vec::with_capacity(items.len());
		let mut entities = Vec::with_capacity(items.len());
		for (i, file) in items.iter().enumerate() {
			indices.push(i);
			entities.push((file.path.to_string_lossy(), file));
		}

		indices.sort_unstable_by(|&a, &b| {
			let promote = self.promote(entities[a].1, entities[b].1);
			if promote != Ordering::Equal {
				promote
			} else if self.reverse {
				natord::compare(&entities[b].0, &entities[a].0)
			} else {
				natord::compare(&entities[a].0, &entities[b].0)
			}
		});

		items.sort_unstable_by_key(|_| indices.pop().unwrap());
	}

	#[inline]
	#[allow(clippy::collapsible_else_if)]
	fn cmp<T: Ord>(&self, a: T, b: T, promote: Ordering) -> Ordering {
		if promote != Ordering::Equal {
			promote
		} else {
			if self.reverse { b.cmp(&a) } else { a.cmp(&b) }
		}
	}

	#[inline]
	fn promote(&self, a: &File, b: &File) -> Ordering {
		if self.dir_first { b.is_dir().cmp(&a.is_dir()) } else { Ordering::Equal }
	}
}
