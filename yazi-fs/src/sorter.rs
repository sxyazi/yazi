use std::cmp::Ordering;

use hashbrown::HashMap;
use yazi_shared::{LcgRng, natsort, translit::Transliterator, url::UrnBuf};

use crate::{File, SortBy, SortByMulti};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
					self.cmp(a.url.ext(), b.url.ext(), self.promote(a, b))
				} else {
					self.cmp_insensitive(
						a.url.ext().map_or(&[], |s| s.as_encoded_bytes()),
						b.url.ext().map_or(&[], |s| s.as_encoded_bytes()),
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

	pub(super) fn sort_multi(&self, items: &mut [File], sizes: &HashMap<UrnBuf, u64>, methods: &[SortBy]) {
		if items.is_empty() || methods.is_empty() {
			return;
		}

		// If only one method, use the existing single-method sort
		if methods.len() == 1 {
			let mut single_sorter = *self;
			single_sorter.by = methods[0];
			single_sorter.sort(items, sizes);
			return;
		}

		items.sort_unstable_by(|a, b| {
			// Try each sorting method in order until we get a non-equal result
			for &sort_method in methods {
				let ordering = self.compare_by_method(a, b, sort_method, sizes);
				if ordering != std::cmp::Ordering::Equal {
					return ordering;
				}
			}
			std::cmp::Ordering::Equal
		});
	}

	fn compare_by_method(&self, a: &File, b: &File, method: SortBy, sizes: &HashMap<UrnBuf, u64>) -> std::cmp::Ordering {
		let promote = self.promote(a, b);
		if promote != std::cmp::Ordering::Equal {
			return promote;
		}

		let ordering = match method {
			SortBy::None => std::cmp::Ordering::Equal,
			SortBy::Mtime => a.mtime.cmp(&b.mtime),
			SortBy::Btime => a.btime.cmp(&b.btime),
			SortBy::Extension => {
				if self.sensitive {
					a.url.ext().cmp(&b.url.ext())
				} else {
					let a_ext = a.url.ext().map_or([].as_slice(), |s| s.as_encoded_bytes());
					let b_ext = b.url.ext().map_or([].as_slice(), |s| s.as_encoded_bytes());
					self.cmp_insensitive_no_promote(a_ext, b_ext)
				}
			}
			SortBy::Alphabetical => {
				if self.sensitive {
					a.urn().encoded_bytes().cmp(b.urn().encoded_bytes())
				} else {
					self.cmp_insensitive_no_promote(a.urn().encoded_bytes(), b.urn().encoded_bytes())
				}
			}
			SortBy::Natural => {
				if self.translit {
					natsort(
						a.urn().encoded_bytes().transliterate().as_bytes(),
						b.urn().encoded_bytes().transliterate().as_bytes(),
						!self.sensitive,
					)
				} else {
					natsort(a.urn().encoded_bytes(), b.urn().encoded_bytes(), !self.sensitive)
				}
			}
			SortBy::Size => {
				let aa = if a.is_dir() { sizes.get(a.urn()).copied() } else { None };
				let bb = if b.is_dir() { sizes.get(b.urn()).copied() } else { None };
				aa.unwrap_or(a.len).cmp(&bb.unwrap_or(b.len))
			}
			SortBy::Random => {
				// For consistent results in multi-sort, we can't use true randomness
				// Instead, use a hash-based comparison for deterministic "randomness"
				use std::collections::hash_map::DefaultHasher;
				use std::hash::{Hash, Hasher};
				
				let mut hasher_a = DefaultHasher::new();
				let mut hasher_b = DefaultHasher::new();
				a.urn().hash(&mut hasher_a);
				b.urn().hash(&mut hasher_b);
				hasher_a.finish().cmp(&hasher_b.finish())
			}
		};

		if self.reverse { ordering.reverse() } else { ordering }
	}

	#[inline(always)]
	fn cmp_insensitive_no_promote(&self, a: &[u8], b: &[u8]) -> std::cmp::Ordering {
		let l = a.len().min(b.len());
		let (lhs, rhs) = if self.reverse { (b, a) } else { (a, b) };

		for i in 0..l.min(lhs.len()).min(rhs.len()) {
			match lhs[i].to_ascii_lowercase().cmp(&rhs[i].to_ascii_lowercase()) {
				std::cmp::Ordering::Equal => (),
				not_eq => return not_eq,
			}
		}

		if self.reverse { b.len().cmp(&a.len()) } else { a.len().cmp(&b.len()) }
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
