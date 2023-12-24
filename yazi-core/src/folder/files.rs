use std::{collections::{BTreeMap, BTreeSet}, mem, ops::Deref, sync::atomic::Ordering};

use anyhow::Result;
use tokio::{fs, select, sync::mpsc::{self, UnboundedReceiver}};
use yazi_config::{manager::SortBy, MANAGER};
use yazi_shared::fs::{File, Url, FILES_TICKET};

use super::FilesSorter;

pub struct Files {
	hidden:              Vec<File>,
	items:               Vec<File>,
	ticket:              u64,
	version:             u64,
	pub(crate) revision: u64,

	pub sizes: BTreeMap<Url, u64>,
	selected:  BTreeSet<Url>,

	sorter:      FilesSorter,
	filter:      Option<String>,
	show_hidden: bool,
}

impl Default for Files {
	fn default() -> Self {
		Self {
			items:    Default::default(),
			hidden:   Default::default(),
			ticket:   Default::default(),
			version:  Default::default(),
			revision: Default::default(),

			sizes:    Default::default(),
			selected: Default::default(),

			sorter:      Default::default(),
			filter:      Default::default(),
			show_hidden: MANAGER.show_hidden,
		}
	}
}

impl Deref for Files {
	type Target = Vec<File>;

	fn deref(&self) -> &Self::Target { &self.items }
}

impl Files {
	pub async fn from_dir(url: &Url) -> Result<UnboundedReceiver<File>> {
		let mut it = fs::read_dir(url).await?;
		let (tx, rx) = mpsc::unbounded_channel();

		tokio::spawn(async move {
			while let Ok(Some(item)) = it.next_entry().await {
				select! {
					_ = tx.closed() => break,
					result = item.metadata() => {
						if let Ok(meta) = result {
							tx.send(File::from_meta(Url::from(item.path()), meta).await).ok();
						}
					}
				}
			}
		});
		Ok(rx)
	}
}

impl Files {
	#[inline]
	pub fn select(&mut self, url: &Url, state: Option<bool>) -> bool {
		let old = self.selected.contains(url);
		let new = state.unwrap_or(!old);

		if new == old {
			return false;
		}

		if new {
			self.selected.insert(url.to_owned());
		} else {
			self.selected.remove(url);
		}
		true
	}

	pub fn select_all(&mut self, state: Option<bool>) -> bool {
		match state {
			Some(true) => {
				let b = if self.selected.len() < self.items.len() {
					true
				} else {
					self.items.iter().any(|f| !self.selected.contains(&f.url))
				};

				self.selected = self.iter().map(|f| f.url()).collect();
				b
			}
			Some(false) => {
				if self.selected.is_empty() {
					return false;
				}

				let b = self.items.iter().any(|f| self.selected.contains(&f.url));
				self.selected.clear();
				b
			}
			None => {
				for item in &self.items {
					if self.selected.contains(&item.url) {
						self.selected.remove(&item.url);
					} else {
						self.selected.insert(item.url());
					}
				}
				!self.items.is_empty()
			}
		}
	}

	pub fn select_index(&mut self, indices: &BTreeSet<usize>, state: Option<bool>) -> bool {
		let mut applied = false;
		let paths: Vec<_> = self.pick(indices).iter().map(|f| f.url()).collect();

		for path in paths {
			applied |= self.select(&path, state);
		}
		applied
	}

	pub fn update_full(&mut self, files: Vec<File>) {
		if files.is_empty() {
			return;
		}

		self.ticket = FILES_TICKET.fetch_add(1, Ordering::Relaxed);
		self.revision += 1;

		(self.hidden, self.items) = if self.show_hidden {
			(vec![], files)
		} else {
			files.into_iter().partition(|f| f.is_hidden())
		};
	}

	pub fn update_part(&mut self, files: Vec<File>, ticket: u64) {
		if !files.is_empty() {
			if ticket != self.ticket {
				return;
			}

			self.revision += 1;
			if self.show_hidden {
				self.hidden.clear();
				self.items.extend(files);
			} else {
				let (hidden, items): (Vec<_>, Vec<_>) = files.into_iter().partition(|f| f.is_hidden());
				self.hidden.extend(hidden);
				self.items.extend(items);
			}
			return;
		}

		self.ticket = ticket;
		self.hidden.clear();
		if !self.items.is_empty() {
			self.revision += 1;
			self.items.clear();
		}
	}

	pub fn update_size(&mut self, sizes: BTreeMap<Url, u64>) {
		if sizes.is_empty() {
			return;
		}

		if self.sorter.by == SortBy::Size {
			self.revision += 1;
		}
		self.sizes.extend(sizes);
	}

	pub fn update_creating(&mut self, files: Vec<File>) {
		if files.is_empty() {
			return;
		}

		macro_rules! go {
			($dist:expr, $src:expr) => {
				let mut todo: BTreeMap<_, _> = $src.into_iter().map(|f| (f.url(), f)).collect();
				for f in &$dist {
					if todo.remove(&f.url).is_some() && todo.is_empty() {
						break;
					}
				}
				if !todo.is_empty() {
					self.revision += 1;
					$dist.extend(todo.into_values());
				}
			};
		}

		let (hidden, items) = if self.show_hidden {
			(vec![], files)
		} else {
			files.into_iter().partition(|f| f.is_hidden())
		};

		if !items.is_empty() {
			go!(self.items, items);
		}
		if !hidden.is_empty() {
			go!(self.hidden, hidden);
		}
	}

	pub fn update_deleting(&mut self, urls: Vec<Url>) {
		if urls.is_empty() {
			return;
		}

		macro_rules! go {
			($dist:expr, $src:expr) => {
				let mut todo: BTreeSet<_> = $src.into_iter().collect();
				let len = $dist.len();

				$dist.retain(|f| !todo.remove(&f.url));
				if $dist.len() != len {
					self.revision += 1;
				}
			};
		}

		let (hidden, items) =
			if self.show_hidden { (vec![], urls) } else { urls.into_iter().partition(|u| u.is_hidden()) };

		if !items.is_empty() {
			go!(self.items, items);
		}
		if !hidden.is_empty() {
			go!(self.hidden, hidden);
		}
	}

	pub fn update_updating(&mut self, files: BTreeMap<Url, File>) -> [BTreeMap<Url, File>; 2] {
		if files.is_empty() {
			return Default::default();
		}

		macro_rules! go {
			($dist:expr, $src:expr) => {
				let len = $src.len();
				for i in 0..$dist.len() {
					if let Some(f) = $src.remove(&$dist[i].url) {
						$dist[i] = f;
						if $src.is_empty() {
							break;
						}
					}
				}
				if $src.len() != len {
					self.revision += 1;
				}
			};
		}

		let (mut hidden, mut items) = if self.show_hidden {
			(BTreeMap::new(), files)
		} else {
			files.into_iter().partition(|(_, f)| f.is_hidden())
		};

		if !items.is_empty() {
			go!(self.items, items);
		}
		if !hidden.is_empty() {
			go!(self.hidden, hidden);
		}
		[hidden, items]
	}

	pub fn update_upserting(&mut self, files: BTreeMap<Url, File>) {
		if files.is_empty() {
			return;
		}

		let [hidden, items] = self.update_updating(files);
		if hidden.is_empty() && items.is_empty() {
			return;
		}

		if !todo.is_empty() {
			go!(self.hidden);
			b |= !removed.is_empty();
		if !hidden.is_empty() {
			self.hidden.extend(hidden.into_values());
		}
		if !items.is_empty() {
			self.revision += 1;
			self.items.extend(items.into_values());
		}
		b
	}

	pub fn catchup_revision(&mut self) -> bool {
		if self.version == self.revision {
			return false;
		}

		self.version = self.revision;
		self.sorter.sort(&mut self.items, &self.sizes);
		true
	}
}

impl Files {
	// --- Items
	pub fn pick(&self, indices: &BTreeSet<usize>) -> Vec<&File> {
		let mut items = Vec::with_capacity(indices.len());
		for (i, item) in self.iter().enumerate() {
			if indices.contains(&i) {
				items.push(item);
			}
		}
		items
	}

	#[inline]
	pub fn position(&self, url: &Url) -> Option<usize> { self.iter().position(|f| &f.url == url) }

	// --- Selected
	pub fn selected(&self, pending: &BTreeSet<usize>, unset: bool) -> Vec<&File> {
		if self.selected.is_empty() && (unset || pending.is_empty()) {
			return Vec::new();
		}

		let selected: BTreeSet<_> = self.selected.iter().collect();
		let pending: BTreeSet<_> =
			pending.iter().filter_map(|&i| self.items.get(i)).map(|f| &f.url).collect();

		let selected: BTreeSet<_> = if unset {
			selected.difference(&pending).cloned().collect()
		} else {
			selected.union(&pending).cloned().collect()
		};

		let mut items = Vec::with_capacity(selected.len());
		for item in &self.items {
			if selected.contains(&item.url) {
				items.push(item);
			}
			if items.len() == selected.len() {
				break;
			}
		}
		items
	}

	#[inline]
	pub fn is_selected(&self, url: &Url) -> bool { self.selected.contains(url) }

	#[inline]
	pub fn has_selected(&self) -> bool {
		if self.selected.is_empty() {
			return false;
		}
		self.iter().any(|f| self.selected.contains(&f.url))
	}

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
	pub fn set_filter(&mut self, keyword: &str) -> bool {
		if keyword == self.filter.as_deref().unwrap_or_default() {
			return false;
		}

		let old_hidden: Vec<File>;
        if self.show_hidden {
            (self.hidden, old_hidden) = (Vec::new(), self.hidden.clone());
        } else {
            (self.hidden, old_hidden) =
                self.hidden.iter().cloned().partition(|f| f.is_hidden());
        };

		if keyword.is_empty() {
			self.filter = None;

			self.items.extend(old_hidden);
			self.sorter.sort(&mut self.items, &self.sizes);

			return true;
		}

		self.filter = Some(keyword.to_owned());

		if !old_hidden.is_empty() {
			self.items.extend(old_hidden);
			self.sorter.sort(&mut self.items, &self.sizes);
		}

		let hidden: Vec<File>;
		(hidden, self.items) = self.items.iter().cloned().partition(|f| {
			let Some(filename) = f.url.file_name().and_then(|o| o.to_str()) else {
				return true;
			};
			!filename.to_lowercase().contains(&keyword.to_lowercase())
		});

		self.hidden.extend(hidden);
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

		self.revision += 1;
		if self.show_hidden {
			self.items.append(&mut self.hidden);
		} else {
			let items = mem::take(&mut self.items);
			(self.hidden, self.items) = items.into_iter().partition(|f| f.is_hidden());
		}
	}
}
