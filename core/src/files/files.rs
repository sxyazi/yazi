use std::{cmp::Ordering, collections::BTreeMap, ops::{Deref, DerefMut}, path::{Path, PathBuf}};

use anyhow::Result;
use config::{manager::SortBy, MANAGER};
use indexmap::IndexMap;
use tokio::fs;

use super::File;

pub struct Files {
	items:           IndexMap<PathBuf, File>,
	pub sort:        FilesSort,
	pub show_hidden: bool,
}

impl Default for Files {
	fn default() -> Self {
		Self {
			items:       Default::default(),
			sort:        Default::default(),
			show_hidden: MANAGER.show_hidden,
		}
	}
}

impl Files {
	pub async fn read(paths: Vec<PathBuf>) -> BTreeMap<PathBuf, File> {
		let mut items = BTreeMap::new();
		for path in paths {
			if let Ok(file) = File::from(&path).await {
				items.insert(path, file);
			}
		}
		items
	}

	pub async fn read_dir(path: &Path) -> Result<BTreeMap<PathBuf, File>> {
		let mut it = fs::read_dir(path).await?;
		let mut items = BTreeMap::new();
		while let Ok(Some(item)) = it.next_entry().await {
			if let Ok(meta) = item.metadata().await {
				let path = item.path();
				let file = File::from_meta(&path, meta).await;
				items.insert(path, file);
			}
		}
		Ok(items)
	}

	#[inline]
	pub fn duplicate(&self, idx: usize) -> Option<File> {
		self.items.get_index(idx).map(|(_, file)| file.clone())
	}

	#[inline]
	pub fn set_sort(&mut self, sort: FilesSort) -> bool {
		if self.sort == sort {
			return false;
		}
		self.sort = sort;
		self.sort()
	}

	pub fn update_read(&mut self, mut items: BTreeMap<PathBuf, File>) -> bool {
		if !self.show_hidden {
			items.retain(|_, item| !item.is_hidden);
		}

		for (path, item) in &mut items {
			if let Some(old) = self.items.get(path) {
				item.is_selected = old.is_selected;

				// Calculate the size of directories is expensive, so we keep the old value,
				// before a new value is calculated and comes to.
				if item.meta.is_dir() {
					item.length = old.length;
				}
			}
		}

		self.items.clear();
		self.items.extend(items);
		self.sort();
		true
	}

	pub fn update_sort(&mut self, mut items: BTreeMap<PathBuf, File>) -> bool {
		for (path, item) in &mut items {
			if let Some(old) = self.items.get(path) {
				item.is_selected = old.is_selected;
			}
		}

		self.items.extend(items);
		self.sort();
		true
	}

	pub fn update_search(&mut self, items: BTreeMap<PathBuf, File>) -> bool {
		if !items.is_empty() {
			self.items.extend(items);
			self.sort();
			return true;
		}

		if !self.items.is_empty() {
			self.items.clear();
			return true;
		}

		false
	}

	fn sort(&mut self) -> bool {
		if self.items.is_empty() {
			return false;
		}

		#[inline]
		#[allow(clippy::collapsible_else_if)]
		fn cmp<T: Ord>(a: T, b: T, reverse: bool, promote: Ordering) -> Ordering {
			if promote != Ordering::Equal {
				promote
			} else {
				if reverse { b.cmp(&a) } else { a.cmp(&b) }
			}
		}

		#[inline]
		fn promote(a: &File, b: &File, dir_first: bool) -> Ordering {
			if dir_first { b.meta.is_dir().cmp(&a.meta.is_dir()) } else { Ordering::Equal }
		}

		let reverse = self.sort.reverse;
		let dir_first = self.sort.dir_first;
		match self.sort.by {
			SortBy::Alphabetical => {
				self.items.sort_by(|_, a, _, b| cmp(&a.path, &b.path, reverse, promote(a, b, dir_first)))
			}
			SortBy::Created => self.items.sort_by(|_, a, _, b| {
				if let (Ok(aa), Ok(bb)) = (a.meta.created(), b.meta.created()) {
					return cmp(aa, bb, reverse, promote(a, b, dir_first));
				}
				Ordering::Equal
			}),
			SortBy::Modified => self.items.sort_by(|_, a, _, b| {
				if let (Ok(aa), Ok(bb)) = (a.meta.modified(), b.meta.modified()) {
					return cmp(aa, bb, reverse, promote(a, b, dir_first));
				}
				Ordering::Equal
			}),
			SortBy::Size => self.items.sort_by(|_, a, _, b| {
				cmp(a.length.unwrap_or(0), b.length.unwrap_or(0), reverse, promote(a, b, dir_first))
			}),
		}
		true
	}
}

impl Deref for Files {
	type Target = IndexMap<PathBuf, File>;

	fn deref(&self) -> &Self::Target { &self.items }
}

impl DerefMut for Files {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.items }
}

#[derive(PartialEq)]
pub struct FilesSort {
	pub by:        SortBy,
	pub reverse:   bool,
	pub dir_first: bool,
}

impl Default for FilesSort {
	fn default() -> Self {
		Self {
			by:        MANAGER.sort_by,
			reverse:   MANAGER.sort_reverse,
			dir_first: MANAGER.sort_dir_first,
		}
	}
}

#[derive(Debug)]
pub enum FilesOp {
	Read(PathBuf, BTreeMap<PathBuf, File>),
	Sort(PathBuf, BTreeMap<PathBuf, File>),
	Search(PathBuf, BTreeMap<PathBuf, File>),
	IOErr(PathBuf),
}

impl FilesOp {
	#[inline]
	pub fn path(&self) -> PathBuf {
		match self {
			Self::Read(path, _) => path,
			Self::Sort(path, _) => path,
			Self::Search(path, _) => path,
			Self::IOErr(path) => path,
		}
		.clone()
	}

	#[inline]
	pub fn read_empty(path: &Path) -> Self { Self::Read(path.to_path_buf(), BTreeMap::new()) }

	#[inline]
	pub fn search_empty(path: &Path) -> Self { Self::Search(path.to_path_buf(), BTreeMap::new()) }
}
