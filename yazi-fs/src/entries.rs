use std::{mem, ops::{Deref, DerefMut, Not}};

use hashbrown::{HashMap, HashSet};
use yazi_shared::{id::Id, path::{PathBufDyn, PathDyn, PathLike}};

use super::{FilesSorter, Filter, IgnoreFilter};
use crate::{SortBy, file::File, op::FILES_TICKET};

#[derive(Default)]
pub struct Entries {
	hidden:       Vec<File>,
	items:        Vec<File>,
	ticket:       Id,
	version:      u64,
	pub revision: u64,

	pub sizes: HashMap<PathBufDyn, u64>,
	pub ranks: HashMap<PathBufDyn, i64>,

	sorter:        FilesSorter,
	filter:        Option<Filter>,
	show_hidden:   bool,
	ignore_filter: Option<IgnoreFilter>,
	show_excluded: bool,
}

impl Deref for Entries {
	type Target = Vec<File>;

	fn deref(&self) -> &Self::Target { &self.items }
}

impl DerefMut for Entries {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.items }
}

impl Entries {
	pub fn new(show_hidden: bool) -> Self { Self { show_hidden, ..Default::default() } }

	pub fn update_full(&mut self, files: Vec<File>) {
		self.ticket = FILES_TICKET.next();

		let (hidden, items) = self.split_files(files);
		if !(items.is_empty() && self.items.is_empty()) {
			self.revision += 1;
		}

		(self.hidden, self.items) = (hidden, items);
	}

	pub fn update_part(&mut self, files: Vec<File>, ticket: Id) {
		if !files.is_empty() {
			if ticket != self.ticket {
				return;
			}

			let (hidden, items) = self.split_files(files);
			if !items.is_empty() {
				self.revision += 1;
			}

			self.hidden.extend(hidden);
			self.items.extend(items);
			return;
		}

		self.ticket = ticket;
		self.hidden.clear();
		if !self.items.is_empty() {
			self.revision += 1;
			self.items.clear();
		}
	}

	pub fn update_size(&mut self, sizes: HashMap<PathBufDyn, u64>) {
		self.sizes.reserve(if self.sizes.is_empty() { sizes.len() } else { sizes.len().div_ceil(2) });

		let mut changed = false;
		for (key, size) in sizes {
			if !key.is_empty() {
				changed |= self.sizes.insert(key, size) != Some(size);
			}
		}

		if changed && self.sorter.by == SortBy::Size {
			self.revision += 1;
		}
	}

	pub fn update_rank(&mut self, mut ranks: HashMap<PathBufDyn, i64>) {
		let mut changed = false;

		if self.ranks.is_empty() {
			ranks.retain(|key, rank| *rank != 0 && !key.is_empty());
			(changed, self.ranks) = (!ranks.is_empty(), mem::take(&mut ranks));
		}

		for (key, rank) in ranks {
			if key.is_empty() {
				continue;
			}

			changed |= if rank == 0 {
				self.ranks.remove(&key).is_some()
			} else {
				self.ranks.insert(key, rank) != Some(rank)
			};
		}

		if changed && self.sorter.by == SortBy::Custom {
			self.revision += 1;
		}
	}

	pub fn update_fail(&mut self) {
		self.ticket = FILES_TICKET.next();
		self.hidden.clear();
		self.items.clear();
	}

	pub fn update_create(&mut self, files: Vec<File>) {
		if files.is_empty() {
			return;
		}

		macro_rules! go {
			($dist:expr, $src:expr, $inc:literal) => {
				let mut todo: HashMap<_, _> = $src.into_iter().map(|f| (f.key().to_owned(), f)).collect();
				for f in &$dist {
					if todo.remove(&f.key()).is_some() && todo.is_empty() {
						break;
					}
				}
				if !todo.is_empty() {
					self.revision += $inc;
					$dist.extend(todo.into_values());
				}
			};
		}

		let (hidden, items) = self.split_files(files);
		if !items.is_empty() {
			go!(self.items, items, 1);
		}
		if !hidden.is_empty() {
			go!(self.hidden, hidden, 0);
		}
	}

	pub fn update_delete(&mut self, mut keys: HashSet<PathBufDyn>) -> Vec<usize> {
		keys.retain(|k| !k.is_empty());
		let mut deleted = Vec::with_capacity(keys.len());

		if !keys.is_empty() {
			let mut i = 0;
			self.items.retain(|f| {
				let b = keys.remove(&f.key());
				if b {
					deleted.push(i)
				}
				i += 1;
				!b
			});
		}

		if !keys.is_empty() {
			self.hidden.retain(|f| !keys.remove(&f.key()));
		}

		self.revision += deleted.is_empty().not() as u64;
		deleted
	}

	pub fn update_existing(
		&mut self,
		mut files: HashMap<PathBufDyn, File>,
	) -> (HashMap<PathBufDyn, File>, HashMap<PathBufDyn, File>) {
		files.retain(|k, f| !k.is_empty() && !f.key().is_empty());
		if files.is_empty() {
			return Default::default();
		}

		macro_rules! go {
			($dist:expr, $src:expr, $inc:literal) => {
				let mut b = true;
				for i in 0..$dist.len() {
					if let Some(f) = $src.remove(&$dist[i].key()) {
						b = b && $dist[i].hits(&f);
						b = b && $dist[i].key() == f.key();

						$dist[i] = f;
						if $src.is_empty() {
							break;
						}
					}
				}
				self.revision += if b { 0 } else { $inc };
			};
		}

		let (mut hidden, mut items) = if let Some(filter) = &self.filter {
			files.into_iter().partition(|(_, f)| self.is_concealed(f) || !filter.matches(f.urn()))
		} else if self.show_hidden && !self.is_excluding() {
			(HashMap::new(), files)
		} else {
			files.into_iter().partition(|(_, f)| self.is_concealed(f))
		};

		if !items.is_empty() {
			go!(self.items, items, 1);
		}
		if !hidden.is_empty() {
			go!(self.hidden, hidden, 0);
		}
		(hidden, items)
	}

	pub fn update_upsert(&mut self, mut files: HashMap<PathBufDyn, File>) {
		files.retain(|k, f| !k.is_empty() && !f.key().is_empty());
		if files.is_empty() {
			return;
		}

		self.update_delete(
			files.iter().filter(|&(k, f)| k != f.key()).map(|(_, f)| f.key().into()).collect(),
		);

		let (hidden, items) = self.update_existing(files);
		if hidden.is_empty() && items.is_empty() {
			return;
		}

		if !hidden.is_empty() {
			self.hidden.extend(hidden.into_values());
		}
		if !items.is_empty() {
			self.revision += 1;
			self.items.extend(items.into_values());
		}
	}

	pub fn catchup_revision(&mut self) -> bool {
		if self.version == self.revision {
			return false;
		}

		self.version = self.revision;
		self.sorter.sort(&mut self.items, &self.sizes, &self.ranks);
		true
	}

	fn split_files(&self, files: impl IntoIterator<Item = File>) -> (Vec<File>, Vec<File>) {
		let files = files.into_iter().filter(|f| !f.key().is_empty());
		if let Some(filter) = &self.filter {
			files.partition(|f| self.is_concealed(f) || !filter.matches(f.urn()))
		} else if self.show_hidden && !self.is_excluding() {
			(vec![], files.collect())
		} else {
			files.partition(|f| self.is_concealed(f))
		}
	}

	fn is_concealed(&self, f: &File) -> bool {
		(f.is_hidden() && !self.show_hidden)
			|| (self.is_excluding() && self.ignore_filter.as_ref().is_some_and(|i| i.matches_url(&f.url)))
	}

	#[inline]
	fn is_excluding(&self) -> bool { self.ignore_filter.is_some() && !self.show_excluded }

	fn resplit(&mut self) {
		let (items, hidden) = (mem::take(&mut self.items), mem::take(&mut self.hidden));
		let (concealed, kept) = self.split_files(items);
		let (still_hidden, revealed) = self.split_files(hidden);

		self.revision += (!concealed.is_empty() || !revealed.is_empty()) as u64;
		self.items = kept;
		self.items.extend(revealed);
		self.hidden = concealed;
		self.hidden.extend(still_hidden);
	}
}

impl Entries {
	// --- Items
	#[inline]
	pub fn all(&self) -> impl Iterator<Item = &File> { self.items.iter().chain(&self.hidden) }

	#[inline]
	pub fn position(&self, key: PathDyn) -> Option<usize> {
		if key.is_empty() { None } else { self.iter().position(|f| f.key() == key) }
	}

	// --- Ticket
	#[inline]
	pub fn ticket(&self) -> Id { self.ticket }

	// --- Sorter
	#[inline]
	pub fn sorter(&self) -> &FilesSorter { &self.sorter }

	pub fn set_sorter(&mut self, sorter: FilesSorter) {
		if self.sorter != sorter {
			self.sorter = sorter;
			self.revision += 1;
		}
	}

	// --- Filter
	#[inline]
	pub fn filter(&self) -> Option<&Filter> { self.filter.as_ref() }

	pub fn set_filter(&mut self, filter: Option<Filter>) -> bool {
		if self.filter == filter {
			return false;
		}

		self.filter = filter;
		if self.filter.is_none() {
			let take = mem::take(&mut self.hidden);
			let (hidden, items) = self.split_files(take);

			self.hidden = hidden;
			if !items.is_empty() {
				self.items.extend(items);
				self.sorter.sort(&mut self.items, &self.sizes, &self.ranks);
			}
			return true;
		}

		let it = mem::take(&mut self.items).into_iter().chain(mem::take(&mut self.hidden));
		(self.hidden, self.items) = self.split_files(it);
		self.sorter.sort(&mut self.items, &self.sizes, &self.ranks);
		true
	}

	// --- Show hidden
	pub fn set_show_hidden(&mut self, state: bool) {
		if mem::replace(&mut self.show_hidden, state) == state {
			return;
		}

		let len = self.items.len();
		let take =
			if self.show_hidden { mem::take(&mut self.hidden) } else { mem::take(&mut self.items) };
		if take.is_empty() {
			return;
		}

		let (hidden, items) = self.split_files(take);
		self.hidden.extend(hidden);
		self.items.extend(items);
		self.revision += (self.items.len() != len) as u64;
	}

	// --- Ignore filter
	pub fn set_ignore_filter(&mut self, filter: Option<IgnoreFilter>) {
		if self.ignore_filter.is_none() && filter.is_none() {
			return;
		}

		self.ignore_filter = filter;
		self.resplit();
	}

	// --- Show excluded
	pub fn set_show_excluded(&mut self, state: bool) {
		if mem::replace(&mut self.show_excluded, state) != state && self.ignore_filter.is_some() {
			self.resplit();
		}
	}
}

#[cfg(test)]
mod tests {
	use std::path::Path;

	use super::*;

	fn file(path: &str) -> File { File::from_dummy(Path::new(path), None) }

	fn names(e: &Entries) -> Vec<String> {
		e.iter().map(|f| f.urn().to_string_lossy().into_owned()).collect()
	}

	fn entries(excluded: &'static str) -> Entries {
		let mut e = Entries::default();
		e.set_sorter(FilesSorter { by: SortBy::Alphabetical, ..Default::default() });
		e.set_ignore_filter(Some(IgnoreFilter::new(move |p| p.ends_with(excluded))));
		e.update_full(["/d/a", "/d/b", "/d/c"].map(file).into());
		e.catchup_revision();
		e
	}

	#[test]
	fn test_swap_ignore_filter_same_count() {
		let mut e = entries("b");
		assert_eq!(names(&e), ["a", "c"]);

		e.set_ignore_filter(Some(IgnoreFilter::new(|p| p.ends_with("a"))));
		assert!(e.catchup_revision());
		assert_eq!(names(&e), ["b", "c"]);
	}

	#[test]
	fn test_show_excluded() {
		let mut e = entries("b");

		e.set_show_excluded(true);
		assert!(e.catchup_revision());
		assert_eq!(names(&e), ["a", "b", "c"]);

		e.set_show_excluded(false);
		assert!(e.catchup_revision());
		assert_eq!(names(&e), ["a", "c"]);
	}
}
