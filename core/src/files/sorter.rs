use std::{cmp::Ordering, collections::BTreeMap, mem};

use config::{manager::SortBy, MANAGER};
use shared::{natsort, Url};

use super::File;

#[derive(Clone, Copy, PartialEq)]
pub struct FilesSorter {
	pub by:        SortBy,
	pub sensitive: bool,
	pub reverse:   bool,
	pub dir_first: bool,
}

impl Default for FilesSorter {
	fn default() -> Self {
		Self {
			by:        MANAGER.sort_by,
			sensitive: MANAGER.sort_sensitive,
			reverse:   MANAGER.sort_reverse,
			dir_first: MANAGER.sort_dir_first,
		}
	}
}

impl FilesSorter {
	pub(super) fn sort(&self, items: &mut Vec<File>, sizes: &BTreeMap<Url, u64>) -> bool {
		if items.is_empty() {
			return false;
		}

		match self.by {
			SortBy::Alphabetical => items.sort_unstable_by(|a, b| {
				if self.sensitive {
					return self.cmp(&*a.url, &*b.url, self.promote(a, b));
				}

				self.cmp(
					a.url.as_os_str().to_ascii_uppercase(),
					b.url.as_os_str().to_ascii_uppercase(),
					self.promote(a, b),
				)
			}),
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
				let aa = if a.is_dir() { sizes.get(&a.url).copied() } else { None };
				let bb = if b.is_dir() { sizes.get(&b.url).copied() } else { None };
				self.cmp(aa.unwrap_or(a.length), bb.unwrap_or(b.length), self.promote(a, b))
			}),
		}
		true
	}

	fn sort_naturally(&self, items: &mut Vec<File>) {
		let mut indices = Vec::with_capacity(items.len());
		let mut entities = Vec::with_capacity(items.len());
		for (i, file) in items.iter().enumerate() {
			indices.push(i);
			entities.push((file.url.to_string_lossy(), file));
		}

		indices.sort_unstable_by(|&a, &b| {
			let promote = self.promote(entities[a].1, entities[b].1);
			if promote != Ordering::Equal {
				return promote;
			}

			let ordering = natsort(&entities[a].0, &entities[b].0, !self.sensitive);
			if self.reverse { ordering.reverse() } else { ordering }
		});

		let dummy = File {
			url:       Default::default(),
			meta:      items[0].meta.clone(),
			length:    0,
			link_to:   None,
			is_link:   false,
			is_hidden: false,
		};

		let mut new = Vec::with_capacity(indices.len());
		for i in indices {
			new.push(mem::replace(&mut items[i], dummy.clone()));
		}
		*items = new;
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
