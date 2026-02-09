use std::cmp::Ordering;

use hashbrown::HashMap;
use rand::{RngCore, SeedableRng, rngs::SmallRng};
use yazi_shared::{natsort, path::PathBufDyn, translit::Transliterator, url::UrlLike};

use crate::{File, SortBy};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FilesSorter {
	pub by:        SortBy,
	pub sensitive: bool,
	pub reverse:   bool,
	pub dir_first: bool,
	pub translit:  bool,
	pub dir_by:    Option<SortBy>,
}

impl FilesSorter {
	pub(super) fn sort(&self, items: &mut [File], sizes: &HashMap<PathBufDyn, u64>) {
		if items.is_empty() {
			return;
		}

		if self.dir_first && self.dir_by.is_some_and(|b| b != self.by) {
			let dir_by = self.dir_by.unwrap();
			// Stable-partition: dirs first, then files
			items.sort_by(|a, b| b.is_dir().cmp(&a.is_dir()));
			let mid = items.iter().position(|f| !f.is_dir()).unwrap_or(items.len());
			let (dirs, files) = items.split_at_mut(mid);

			// Sort dirs with dir_by, non-reversed (already partitioned)
			let dir_sorter =
				Self { by: dir_by, reverse: false, dir_first: false, dir_by: None, ..*self };
			dir_sorter.sort_with_by(dirs, sizes);

			// Sort files with the main sort_by
			let file_sorter = Self { dir_first: false, dir_by: None, ..*self };
			file_sorter.sort_with_by(files, sizes);
			return;
		}

		self.sort_with_by(items, sizes);
	}

	fn sort_with_by(&self, items: &mut [File], sizes: &HashMap<PathBufDyn, u64>) {
		if items.is_empty() {
			return;
		}

		let by_alphabetical = |a: &File, b: &File| {
			if self.sensitive {
				self.cmp(a.urn().encoded_bytes(), b.urn().encoded_bytes(), self.promote(a, b))
			} else {
				self.cmp_insensitive(a.urn().encoded_bytes(), b.urn().encoded_bytes(), self.promote(a, b))
			}
		};

		match self.by {
			SortBy::None => {}
			SortBy::Mtime => items.sort_unstable_by(|a, b| {
				let ord = self.cmp(a.mtime, b.mtime, self.promote(a, b));
				if ord == Ordering::Equal { by_alphabetical(a, b) } else { ord }
			}),
			SortBy::Btime => items.sort_unstable_by(|a, b| {
				let ord = self.cmp(a.btime, b.btime, self.promote(a, b));
				if ord == Ordering::Equal { by_alphabetical(a, b) } else { ord }
			}),
			SortBy::Extension => items.sort_unstable_by(|a, b| {
				let aa = a.url.ext().filter(|_| a.is_file());
				let bb = b.url.ext().filter(|_| b.is_file());
				let ord = if self.sensitive {
					self.cmp(aa, bb, self.promote(a, b))
				} else {
					self.cmp_insensitive(
						aa.map_or(&[], |s| s.encoded_bytes()),
						bb.map_or(&[], |s| s.encoded_bytes()),
						self.promote(a, b),
					)
				};
				if ord == Ordering::Equal { by_alphabetical(a, b) } else { ord }
			}),
			SortBy::Alphabetical => items.sort_unstable_by(by_alphabetical),
			SortBy::Natural => self.sort_naturally(items),
			SortBy::Size => items.sort_unstable_by(|a, b| {
				let aa = if a.is_dir() { sizes.get(&a.urn()).copied() } else { None };
				let bb = if b.is_dir() { sizes.get(&b.urn()).copied() } else { None };
				let ord = self.cmp(aa.unwrap_or(a.len), bb.unwrap_or(b.len), self.promote(a, b));
				if ord == Ordering::Equal { by_alphabetical(a, b) } else { ord }
			}),
			SortBy::Random => {
				let mut rng = SmallRng::from_os_rng();
				items.sort_unstable_by(|a, b| self.cmp(rng.next_u64(), rng.next_u64(), self.promote(a, b)))
			}
		}
	}

	fn sort_naturally(&self, items: &mut [File]) {
		items.sort_unstable_by(|a, b| {
			let promote = self.promote(a, b);
			if promote != Ordering::Equal {
				return promote;
			}

			let ordering = if self.translit {
				natsort(
					a.urn().encoded_bytes().transliterate().as_bytes(),
					b.urn().encoded_bytes().transliterate().as_bytes(),
					!self.sensitive,
				)
			} else {
				natsort(a.urn().encoded_bytes(), b.urn().encoded_bytes(), !self.sensitive)
			};

			if self.reverse { ordering.reverse() } else { ordering }
		});
	}

	#[inline(always)]
	#[allow(clippy::collapsible_else_if)]
	fn cmp<T: Ord>(&self, a: T, b: T, promote: Ordering) -> Ordering {
		if promote != Ordering::Equal {
			promote
		} else {
			if self.reverse { b.cmp(&a) } else { a.cmp(&b) }
		}
	}

	#[inline(always)]
	fn cmp_insensitive(&self, a: &[u8], b: &[u8], promote: Ordering) -> Ordering {
		if promote != Ordering::Equal {
			return promote;
		}

		let l = a.len().min(b.len());
		let (lhs, rhs) = if self.reverse { (&b[..l], &a[..l]) } else { (&a[..l], &b[..l]) };

		for i in 0..l {
			match lhs[i].to_ascii_lowercase().cmp(&rhs[i].to_ascii_lowercase()) {
				Ordering::Equal => (),
				not_eq => return not_eq,
			}
		}

		if self.reverse { b.len().cmp(&a.len()) } else { a.len().cmp(&b.len()) }
	}

	#[inline(always)]
	fn promote(&self, a: &File, b: &File) -> Ordering {
		if self.dir_first { b.is_dir().cmp(&a.is_dir()) } else { Ordering::Equal }
	}
}
