use std::cmp::Ordering;

use hashbrown::HashMap;
use yazi_shared::{LcgRng, natsort, translit::Transliterator, url::UrnBuf};

use crate::{File, SortBy, SortBys};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FilesSorter {
	pub by: SortBys,
	pub sensitive: bool,
	pub reverse: bool,
	pub dir_first: bool,
	pub translit: bool,
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

		let by_natural = |a: &File, b: &File| {
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
		};

		items.sort_unstable_by(|a, b| {
			for key in &self.by.0 {
				let ord = match key {
					SortBy::None => Ordering::Equal,
					SortBy::Mtime => self.cmp(a.mtime, b.mtime, self.promote(a, b)),
					SortBy::Btime => self.cmp(a.btime, b.btime, self.promote(a, b)),
					SortBy::Extension => {
						if self.sensitive {
							self.cmp(a.url.ext(), b.url.ext(), self.promote(a, b))
						} else {
							self.cmp_insensitive(
								a.url.ext().map_or(&[], |s| s.as_encoded_bytes()),
								b.url.ext().map_or(&[], |s| s.as_encoded_bytes()),
								self.promote(a, b),
							)
						}
					}
					SortBy::Alphabetical => by_alphabetical(a, b),
					SortBy::Natural => by_natural(a, b),
					SortBy::Size => {
						let aa = if a.is_dir() { sizes.get(a.urn()).copied() } else { None };
						let bb = if b.is_dir() { sizes.get(b.urn()).copied() } else { None };
						self.cmp(aa.unwrap_or(a.len), bb.unwrap_or(b.len), self.promote(a, b))
					}
					SortBy::Random => {
						let mut rng = LcgRng::default();
						self.cmp(rng.next(), rng.next(), self.promote(a, b))
					}
				};

				if ord != Ordering::Equal {
					return ord;
				}
			}

			// fallback to alphabetical sorting if all other keys are equal
			by_alphabetical(a, b)
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
