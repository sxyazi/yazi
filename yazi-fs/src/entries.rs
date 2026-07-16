use std::{mem, ops::{Deref, DerefMut, Not}};

use hashbrown::{HashMap, HashSet};
use yazi_shared::{id::Id, path::{PathBufDyn, PathDyn, PathLike}};

use super::{FilesSorter, Filter};
use crate::{FILES_TICKET, SortBy, file::File};

#[derive(Default)]
pub struct Entries {
	hidden:       Vec<File>,
	items:        Vec<File>,
	ticket:       Id,
	version:      u64,
	pub revision: u64,

	pub sizes: HashMap<PathBufDyn, u64>,

	sorter:      FilesSorter,
	filter:      Option<Filter>,
	show_hidden: bool,
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

	pub fn update_ioerr(&mut self) {
		self.ticket = FILES_TICKET.next();
		self.hidden.clear();
		self.items.clear();
	}

	pub fn update_creating(&mut self, files: Vec<File>) {
		if files.is_empty() {
			return;
		}

		macro_rules! go {
			($dist:expr, $src:expr, $inc:literal) => {
				let mut todo: HashMap<_, _> =
					$src.into_iter().map(|f| (f.entry_key().to_owned(), f)).collect();
				for f in &$dist {
					if todo.remove(&f.entry_key()).is_some() && todo.is_empty() {
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

	pub fn update_deleting(&mut self, mut keys: HashSet<PathBufDyn>) -> Vec<usize> {
		keys.retain(|k| !k.is_empty());
		let mut deleted = Vec::with_capacity(keys.len());

		if !keys.is_empty() {
			let mut i = 0;
			self.items.retain(|f| {
				let b = keys.remove(&f.entry_key());
				if b {
					deleted.push(i)
				}
				i += 1;
				!b
			});
		}

		if !keys.is_empty() {
			self.hidden.retain(|f| !keys.remove(&f.entry_key()));
		}

		self.revision += deleted.is_empty().not() as u64;
		deleted
	}

	pub fn update_updating(
		&mut self,
		mut files: HashMap<PathBufDyn, File>,
	) -> (HashMap<PathBufDyn, File>, HashMap<PathBufDyn, File>) {
		files.retain(|k, f| !k.is_empty() && !f.entry_key().is_empty());
		if files.is_empty() {
			return Default::default();
		}

		macro_rules! go {
			($dist:expr, $src:expr, $inc:literal) => {
				let mut b = true;
				for i in 0..$dist.len() {
					if let Some(f) = $src.remove(&$dist[i].entry_key()) {
						b = b && $dist[i].cha.hits(f.cha);
						b = b && $dist[i].entry_key() == f.entry_key();

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
			files
				.into_iter()
				.partition(|(_, f)| (f.is_hidden() && !self.show_hidden) || !filter.matches(f.urn()))
		} else if self.show_hidden {
			(HashMap::new(), files)
		} else {
			files.into_iter().partition(|(_, f)| f.is_hidden())
		};

		if !items.is_empty() {
			go!(self.items, items, 1);
		}
		if !hidden.is_empty() {
			go!(self.hidden, hidden, 0);
		}
		(hidden, items)
	}

	pub fn update_upserting(&mut self, mut files: HashMap<PathBufDyn, File>) {
		files.retain(|k, f| !k.is_empty() && !f.entry_key().is_empty());
		if files.is_empty() {
			return;
		}

		self.update_deleting(
			files
				.iter()
				.filter(|&(k, f)| k != f.entry_key())
				.map(|(_, f)| f.entry_key().into())
				.collect(),
		);

		let (hidden, items) = self.update_updating(files);
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
		self.sorter.sort(&mut self.items, &self.sizes);
		true
	}

	fn split_files(&self, files: impl IntoIterator<Item = File>) -> (Vec<File>, Vec<File>) {
		let files = files.into_iter().filter(|f| !f.entry_key().is_empty());
		if let Some(filter) = &self.filter {
			files.partition(|f| (f.is_hidden() && !self.show_hidden) || !filter.matches(f.urn()))
		} else if self.show_hidden {
			(vec![], files.collect())
		} else {
			files.partition(|f| f.is_hidden())
		}
	}
}

impl Entries {
	// --- Items
	#[inline]
	pub fn position(&self, key: PathDyn) -> Option<usize> {
		if key.is_empty() { None } else { self.iter().position(|f| f.entry_key() == key) }
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
				self.sorter.sort(&mut self.items, &self.sizes);
			}
			return true;
		}

		let it = mem::take(&mut self.items).into_iter().chain(mem::take(&mut self.hidden));
		(self.hidden, self.items) = self.split_files(it);
		self.sorter.sort(&mut self.items, &self.sizes);
		true
	}

	// --- Show hidden
	pub fn set_show_hidden(&mut self, state: bool) {
		if self.show_hidden == state {
			return;
		}

		self.show_hidden = state;
		if self.show_hidden && self.hidden.is_empty() {
			return;
		} else if !self.show_hidden && self.items.is_empty() {
			return;
		}

		let take =
			if self.show_hidden { mem::take(&mut self.hidden) } else { mem::take(&mut self.items) };
		let (hidden, items) = self.split_files(take);

		self.hidden.extend(hidden);
		if !items.is_empty() {
			self.revision += 1;
			self.items.extend(items);
		}
	}
}
