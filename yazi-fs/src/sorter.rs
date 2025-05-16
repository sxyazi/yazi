use std::{cmp::Ordering, collections::HashMap};

use yazi_shared::{LcgRng, natsort, translit::Transliterator, url::UrnBuf};

use crate::{File, SortBy};

#[derive(Clone, Copy, Default, PartialEq)]
pub struct FilesSorter {
	pub by:        SortBy,
	pub sensitive: bool,
	pub reverse:   bool,
	pub dir_first: bool,
	pub translit:  bool,
}

impl FilesSorter {
	pub(super) fn sort(&self, items: &mut [File], sizes: &HashMap<UrnBuf, u64>) {
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
				let ord = if self.sensitive {
					self.cmp(a.url.extension(), b.url.extension(), self.promote(a, b))
				} else {
					self.cmp_insensitive(
						a.url.extension().map_or(&[], |s| s.as_encoded_bytes()),
						b.url.extension().map_or(&[], |s| s.as_encoded_bytes()),
						self.promote(a, b),
					)
				};
				if ord == Ordering::Equal { by_alphabetical(a, b) } else { ord }
			}),
			SortBy::Alphabetical => items.sort_unstable_by(by_alphabetical),
			SortBy::Natural => self.sort_naturally(items),
			SortBy::Size => items.sort_unstable_by(|a, b| {
				let aa = if a.is_dir() { sizes.get(a.urn()).copied() } else { None };
				let bb = if b.is_dir() { sizes.get(b.urn()).copied() } else { None };
				let ord = self.cmp(aa.unwrap_or(a.len), bb.unwrap_or(b.len), self.promote(a, b));
				if ord == Ordering::Equal { by_alphabetical(a, b) } else { ord }
			}),
			SortBy::Random => {
				let mut rng = LcgRng::default();
				items.sort_unstable_by(|a, b| self.cmp(rng.next(), rng.next(), self.promote(a, b)))
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
