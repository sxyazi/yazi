use std::cmp::Ordering;

use hashbrown::HashMap;
use rand::{RngCore, SeedableRng, rngs::SmallRng};
use yazi_shared::{natsort, path::PathBufDyn, translit::Transliterator, url::UrlLike};

use crate::{File, SortBy, SortFallback};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FilesSorter {
	pub by:        SortBy,
	pub sensitive: bool,
	pub reverse:   bool,
	pub dir_first: bool,
	pub translit:  bool,
	pub fallback:  SortFallback,
}

impl FilesSorter {
	pub(super) fn sort(&self, items: &mut [File], sizes: &HashMap<PathBufDyn, u64>) {
		if items.is_empty() {
			return;
		}

		macro_rules! promote {
			($a:ident, $b:ident) => {
				if self.dir_first {
					match $b.is_dir().cmp(&$a.is_dir()) {
						Ordering::Equal => {}
						not_eq => return not_eq,
					}
				}
			};
		}

		match self.by {
			SortBy::None => {}
			SortBy::Mtime => items.sort_unstable_by(|a, b| {
				promote!(a, b);
				self.fallback(a, b, self.cmp(a.mtime, b.mtime))
			}),
			SortBy::Btime => items.sort_unstable_by(|a, b| {
				promote!(a, b);
				self.fallback(a, b, self.cmp(a.btime, b.btime))
			}),
			SortBy::Extension => items.sort_unstable_by(|a, b| {
				promote!(a, b);
				let aa = a.url.ext().filter(|_| a.is_file());
				let bb = b.url.ext().filter(|_| b.is_file());
				let ord = if self.sensitive {
					self.cmp(aa, bb)
				} else {
					self.cmp_insensitive(
						aa.map_or(&[], |s| s.encoded_bytes()),
						bb.map_or(&[], |s| s.encoded_bytes()),
					)
				};
				self.fallback(a, b, ord)
			}),
			SortBy::Alphabetical => items.sort_unstable_by(|a, b| {
				promote!(a, b);
				self.sort_alphabetically(a, b)
			}),
			SortBy::Natural => items.sort_unstable_by(|a, b| {
				promote!(a, b);
				self.sort_naturally(a, b)
			}),
			SortBy::Size => items.sort_unstable_by(|a, b| {
				promote!(a, b);
				let aa = if a.is_dir() { sizes.get(&a.urn()).copied() } else { None };
				let bb = if b.is_dir() { sizes.get(&b.urn()).copied() } else { None };
				self.fallback(a, b, self.cmp(aa.unwrap_or(a.len), bb.unwrap_or(b.len)))
			}),
			SortBy::Random => {
				let mut rng = SmallRng::from_os_rng();
				items.sort_unstable_by(|a, b| {
					promote!(a, b);
					self.cmp(rng.next_u64(), rng.next_u64())
				})
			}
		}
	}

	#[inline(always)]
	fn sort_alphabetically(&self, a: &File, b: &File) -> Ordering {
		if self.sensitive {
			self.cmp(a.urn().encoded_bytes(), b.urn().encoded_bytes())
		} else {
			self.cmp_insensitive(a.urn().encoded_bytes(), b.urn().encoded_bytes())
		}
	}

	#[inline(always)]
	fn sort_naturally(&self, a: &File, b: &File) -> Ordering {
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
	}

	#[inline(always)]
	fn fallback(&self, a: &File, b: &File, ord: Ordering) -> Ordering {
		if ord != Ordering::Equal {
			return ord;
		}

		match self.fallback {
			SortFallback::Alphabetical => self.sort_alphabetically(a, b),
			SortFallback::Natural => self.sort_naturally(a, b),
		}
	}

	#[inline(always)]
	fn cmp<T: Ord>(&self, a: T, b: T) -> Ordering { if self.reverse { b.cmp(&a) } else { a.cmp(&b) } }

	#[inline(always)]
	fn cmp_insensitive(&self, a: &[u8], b: &[u8]) -> Ordering {
		let l = a.len().min(b.len());
		let (lhs, rhs) = if self.reverse { (&b[..l], &a[..l]) } else { (&a[..l], &b[..l]) };

		for i in 0..l {
			match lhs[i].to_ascii_lowercase().cmp(&rhs[i].to_ascii_lowercase()) {
				Ordering::Equal => {}
				not_eq => return not_eq,
			}
		}

		if self.reverse { b.len().cmp(&a.len()) } else { a.len().cmp(&b.len()) }
	}
}
