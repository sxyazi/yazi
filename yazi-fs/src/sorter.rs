use std::cmp::Ordering;

use hashbrown::HashMap;
use rand::{Rng, make_rng, rngs::SmallRng};
use yazi_shared::{natsort, path::PathBufDyn, translit::Transliterator, url::UrlLike};

use crate::{SortBy, SortFallback, file::File};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FilesSorter {
	pub by:          SortBy,
	pub sensitive:   bool,
	pub reverse:     bool,
	pub dir_first:   bool,
	pub hidden_last: bool,
	pub translit:    bool,
	pub fallback:    SortFallback,
}

impl FilesSorter {
	pub(super) fn sort(
		&self,
		items: &mut [File],
		sizes: &HashMap<PathBufDyn, u64>,
		ranks: &HashMap<PathBufDyn, i64>,
	) {
		if items.is_empty() {
			return;
		}

		macro_rules! promote {
			($a:ident, $b:ident) => {
				if self.groups_hidden() {
					match $a.is_hidden().cmp(&$b.is_hidden()) {
						Ordering::Equal => {}
						not_eq => return not_eq,
					}
				}
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
			SortBy::Mtime => items.sort_by(|a, b| {
				promote!(a, b);
				self.fallback(a, b, self.cmp(a.mtime, b.mtime))
			}),
			SortBy::Btime => items.sort_by(|a, b| {
				promote!(a, b);
				self.fallback(a, b, self.cmp(a.btime, b.btime))
			}),
			SortBy::Extension => items.sort_by(|a, b| {
				promote!(a, b);
				let aa = a.ext().filter(|_| a.is_file());
				let bb = b.ext().filter(|_| b.is_file());
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
			SortBy::Alphabetical => items.sort_by(|a, b| {
				promote!(a, b);
				self.fallback(a, b, self.sort_alphabetically(a, b))
			}),
			SortBy::Natural => items.sort_by(|a, b| {
				promote!(a, b);
				self.fallback(a, b, self.sort_naturally(a, b))
			}),
			SortBy::Size => items.sort_by(|a, b| {
				promote!(a, b);
				let aa = if a.is_dir() { sizes.get(&a.key()).copied() } else { None };
				let bb = if b.is_dir() { sizes.get(&b.key()).copied() } else { None };
				self.fallback(a, b, self.cmp(aa.unwrap_or(a.len), bb.unwrap_or(b.len)))
			}),
			SortBy::Random => {
				let mut rng = make_rng::<SmallRng>();
				items.sort_by(|a, b| {
					promote!(a, b);
					self.cmp(rng.next_u64(), rng.next_u64())
				})
			}
			SortBy::Custom => items.sort_by(|a, b| {
				promote!(a, b);
				let aa = ranks.get(&a.key()).copied().unwrap_or_default();
				let bb = ranks.get(&b.key()).copied().unwrap_or_default();
				self.fallback(a, b, self.cmp(aa, bb))
			}),
		}
	}

	#[inline(always)]
	fn groups_hidden(&self) -> bool { self.hidden_last && self.by != SortBy::Random }

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
			SortFallback::Alphabetical => self.cmp(a.urn().encoded_bytes(), b.urn().encoded_bytes()),
			SortFallback::Natural => {
				let ord = natsort(a.urn().encoded_bytes(), b.urn().encoded_bytes(), false);
				if self.reverse { ord.reverse() } else { ord }
			}
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

#[cfg(test)]
mod tests {
	use std::time::{Duration, UNIX_EPOCH};

	use super::*;
	use crate::stat::StatType;

	fn file(name: &str, dir: bool) -> File {
		File::from_dummy(
			std::path::PathBuf::from(format!("/tmp/{name}")),
			Some(if dir { StatType::Dir } else { StatType::File }),
		)
	}

	fn fixture_t1() -> Vec<File> {
		["Documents", "src", "README.md", "notes.txt", ".config", ".git", ".zshrc"]
			.map(|n| file(n, matches!(n, "Documents" | "src" | ".config" | ".git")))
			.to_vec()
	}

	fn sorter(by: SortBy, reverse: bool, dir_first: bool) -> FilesSorter {
		FilesSorter { by, reverse, dir_first, hidden_last: true, ..Default::default() }
	}

	fn run(items: &mut [File], s: &FilesSorter) {
		s.sort(items, &Default::default(), &Default::default());
	}

	fn names(items: &[File]) -> Vec<String> {
		items.iter().map(|f| f.urn().to_string_lossy().into_owned()).collect()
	}

	// T1: visible dirs, visible files, hidden dirs, hidden files
	#[test]
	fn t1_natural_grouping() {
		let mut items = fixture_t1();
		run(&mut items, &sorter(SortBy::Natural, false, true));
		assert_eq!(names(&items), [
			"Documents",
			"src",
			"notes.txt",
			"README.md",
			".config",
			".git",
			".zshrc"
		]);
	}

	// T2: reverse flips within groups only, hidden blocks stay last
	#[test]
	fn t2_reverse_within_groups() {
		let mut items = fixture_t1();
		run(&mut items, &sorter(SortBy::Natural, true, true));
		assert_eq!(names(&items), [
			"src",
			"Documents",
			"README.md",
			"notes.txt",
			".git",
			".config",
			".zshrc"
		]);
	}

	// T3: mtime sort keeps grouping regardless of mtimes, both directions
	#[test]
	fn t3_mtime_grouping() {
		let mut newest = file(".cache", true);
		newest.stat.mtime = Some(UNIX_EPOCH + Duration::from_secs(300));
		let mut oldest = file("a.txt", false);
		oldest.stat.mtime = Some(UNIX_EPOCH + Duration::from_secs(100));
		let mut newest_visible = file("b.txt", false);
		newest_visible.stat.mtime = Some(UNIX_EPOCH + Duration::from_secs(200));

		for reverse in [false, true] {
			let mut items = vec![newest.clone(), oldest.clone(), newest_visible.clone()];
			run(&mut items, &sorter(SortBy::Mtime, reverse, true));
			let n = names(&items);
			assert_eq!(n.last().unwrap(), ".cache", "newest hidden dir must stay last");
			assert!(n.contains(&"b.txt".to_string()) && n.contains(&"a.txt".to_string()));
			assert_eq!(n.iter().filter(|x| !x.starts_with('.')).count(), 2);
			assert_eq!(n[0], if reverse { "b.txt" } else { "a.txt" });
		}
	}

	// T4: dir_first=false collapses to visible -> hidden, types mixed
	#[test]
	fn t4_dir_first_false() {
		let mut items =
			[".git", "README.md", ".zshrc", "src"].map(|n| file(n, matches!(n, ".git" | "src"))).to_vec();
		run(&mut items, &sorter(SortBy::Natural, false, false));
		assert_eq!(names(&items), ["README.md", "src", ".git", ".zshrc"]);
	}

	// T5: random ignores hidden grouping (guard function, not list order)
	#[test]
	fn t5_random_ignores_grouping() {
		assert!(sorter(SortBy::Natural, false, true).groups_hidden());
		assert!(sorter(SortBy::Size, true, false).groups_hidden());
		assert!(!sorter(SortBy::Random, false, true).groups_hidden());
	}

	// T6: with no hidden entries (show_hidden=false case), key is inert
	#[test]
	fn t6_no_hidden_inert() {
		let visible: Vec<File> = ["Documents", "src", "README.md", "notes.txt"]
			.map(|n| file(n, n == "Documents" || n == "src"))
			.to_vec();

		let mut a = visible.clone();
		run(&mut a, &sorter(SortBy::Natural, false, true));
		let mut b = visible;
		run(&mut b, &FilesSorter { by: SortBy::Natural, dir_first: true, ..Default::default() });
		assert_eq!(names(&a), names(&b));
	}

	// T7: hidden_last=false is byte-for-byte upstream behavior
	#[test]
	fn t7_disabled_matches_upstream() {
		let mut items = fixture_t1();
		run(&mut items, &FilesSorter { by: SortBy::Natural, dir_first: true, ..Default::default() });
		assert_eq!(names(&items), [
			".config",
			".git",
			"Documents",
			"src",
			".zshrc",
			"notes.txt",
			"README.md"
		]);
	}

	// T8: global invariant across every deterministic sort mode and reverse flag
	#[test]
	fn t8_visible_before_hidden_invariant() {
		for by in [
			SortBy::None,
			SortBy::Mtime,
			SortBy::Btime,
			SortBy::Extension,
			SortBy::Alphabetical,
			SortBy::Natural,
			SortBy::Size,
			SortBy::Custom,
		] {
			for reverse in [false, true] {
				let mut items = fixture_t1();
				run(&mut items, &sorter(by, reverse, true));
				let max_visible = items
					.iter()
					.position(|f| !f.is_hidden())
					.map(|_| items.iter().rposition(|f| !f.is_hidden()).unwrap())
					.unwrap();
				let min_hidden = items.iter().position(|f| f.is_hidden());
				if let Some(min_hidden) = min_hidden {
					assert!(
						max_visible < min_hidden,
						"violated: {by:?} reverse={reverse}: {:?}",
						names(&items)
					);
				}
			}
		}
	}
}
