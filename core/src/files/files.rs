use std::{collections::{BTreeMap, BTreeSet}, mem, ops::Deref, sync::atomic::Ordering};

use anyhow::Result;
use config::manager::SortBy;
use shared::Url;
use tokio::{fs, select, sync::mpsc::{self, UnboundedReceiver}};

use super::{File, FilesSorter, FILES_VERSION};

pub struct Files {
	items:   Vec<File>,
	hidden:  Vec<File>,
	version: u64,

	sizes:    BTreeMap<Url, u64>,
	selected: BTreeSet<Url>,

	sorter:      FilesSorter,
	show_hidden: bool,
}

impl Default for Files {
	fn default() -> Self {
		Self {
			items:   Default::default(),
			hidden:  Default::default(),
			version: Default::default(),

			sizes:    Default::default(),
			selected: Default::default(),

			sorter:      Default::default(),
			show_hidden: true,
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
					Ok(meta) = item.metadata() => {
						tx.send(File::from_meta(Url::from(item.path()), meta).await).ok();
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
		let new = if let Some(new) = state { new } else { !old };

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

				self.selected = self.iter().map(|f| f.url_owned()).collect();
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
						self.selected.insert(item.url_owned());
					}
				}
				!self.items.is_empty()
			}
		}
	}

	pub fn select_index(&mut self, indices: &BTreeSet<usize>, state: Option<bool>) -> bool {
		let mut applied = false;
		let paths: Vec<_> = self.pick(indices).iter().map(|f| f.url_owned()).collect();

		for path in paths {
			applied |= self.select(&path, state);
		}
		applied
	}

	pub fn update_full(&mut self, mut items: Vec<File>) -> bool {
		if !self.show_hidden {
			(self.hidden, items) = items.into_iter().partition(|f| f.is_hidden);
		}
		self.sorter.sort(&mut items, &self.sizes);
		self.items = items;
		self.version = FILES_VERSION.fetch_add(1, Ordering::Relaxed);
		true
	}

	pub fn update_part(&mut self, version: u64, items: Vec<File>) -> bool {
		if !items.is_empty() {
			if version != self.version {
				return false;
			}

			if self.show_hidden {
				self.items.extend(items);
			} else {
				let (hidden, items): (Vec<_>, Vec<_>) = items.into_iter().partition(|f| f.is_hidden);
				self.items.extend(items);
				self.hidden.extend(hidden);
			}

			self.sorter.sort(&mut self.items, &self.sizes);
			return true;
		}

		self.version = version;
		if !self.items.is_empty() {
			self.items.clear();
			self.hidden.clear();
			return true;
		}
		false
	}

	pub fn update_size(&mut self, items: BTreeMap<Url, u64>) -> bool {
		self.sizes.extend(items);
		if self.sorter.by == SortBy::Size {
			self.sorter.sort(&mut self.items, &self.sizes);
		}
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

	#[inline]
	pub fn duplicate(&self, idx: usize) -> Option<File> { self.items.get(idx).cloned() }

	// --- Size
	#[inline]
	pub fn size(&self, url: &Url) -> Option<u64> { self.sizes.get(url).copied() }

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
	pub fn sorter(&self) -> &FilesSorter { &self.sorter }

	#[inline]
	pub fn set_sorter(&mut self, sorter: FilesSorter) -> bool {
		if self.sorter == sorter {
			return false;
		}
		self.sorter = sorter;
		self.sorter.sort(&mut self.items, &self.sizes)
	}

	// --- Show hidden
	#[inline]
	pub fn set_show_hidden(&mut self, state: bool) -> bool {
		if state == self.show_hidden {
			return false;
		} else if state && self.hidden.is_empty() {
			return false;
		}

		if state {
			self.items.append(&mut self.hidden);
			self.sorter.sort(&mut self.items, &self.sizes);
		} else {
			let items = mem::take(&mut self.items);
			(self.hidden, self.items) = items.into_iter().partition(|f| f.is_hidden);
		}

		self.show_hidden = state;
		true
	}
}
