use std::{ops::{Deref, DerefMut}, path::{Path, PathBuf}};

use anyhow::Result;
use indexmap::IndexMap;
use tokio::fs;

use super::File;
use crate::config::{manager::SortBy, MANAGER};

#[derive(Default)]
pub struct Files {
	items:           IndexMap<PathBuf, File>,
	sort:            FilesSort,
	pub show_hidden: bool,
}

impl Files {
	pub async fn read(paths: Vec<PathBuf>) -> IndexMap<PathBuf, File> {
		let mut items = IndexMap::new();
		for path in paths {
			if let Ok(file) = File::from(&path).await {
				items.insert(path, file);
			}
		}
		items
	}

	pub async fn read_dir(path: &Path) -> Result<IndexMap<PathBuf, File>> {
		let mut it = fs::read_dir(path).await?;
		let mut items = IndexMap::new();
		while let Ok(Some(item)) = it.next_entry().await {
			if let Ok(meta) = item.metadata().await {
				let path = item.path();
				let file = File::from_meta(&path, meta).await;
				items.insert(path, file);
			}
		}
		Ok(items)
	}

	pub fn sort(&mut self) {
		fn cmp<T: Ord>(a: T, b: T, reverse: bool) -> std::cmp::Ordering {
			if reverse { b.cmp(&a) } else { a.cmp(&b) }
		}

		let reverse = self.sort.reverse;
		match self.sort.by {
			SortBy::Alphabetical => self.items.sort_by(|_, a, _, b| cmp(&a.path, &b.path, reverse)),
			SortBy::Created => self.items.sort_by(|_, a, _, b| {
				if let (Ok(a), Ok(b)) = (a.meta.created(), b.meta.created()) {
					return cmp(a, b, reverse);
				}
				std::cmp::Ordering::Equal
			}),
			SortBy::Modified => self.items.sort_by(|_, a, _, b| {
				if let (Ok(a), Ok(b)) = (a.meta.modified(), b.meta.modified()) {
					return cmp(a, b, reverse);
				}
				std::cmp::Ordering::Equal
			}),
			SortBy::Size => {
				self.items.sort_by(|_, a, _, b| cmp(a.length.unwrap_or(0), b.length.unwrap_or(0), reverse))
			}
		}
	}

	#[inline]
	pub fn duplicate(&self, idx: usize) -> Option<File> {
		self.items.get_index(idx).map(|(_, file)| file.clone())
	}

	pub fn update_read(&mut self, mut items: IndexMap<PathBuf, File>) -> bool {
		if !self.show_hidden {
			items.retain(|_, item| !item.is_hidden);
		}

		for (path, item) in &mut items {
			if let Some(old) = self.items.get(path) {
				item.length = old.length;
				item.is_selected = old.is_selected;
			}
		}

		self.items = items;
		self.sort();
		true
	}

	pub fn update_search(&mut self, items: IndexMap<PathBuf, File>) -> bool {
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
}

impl Deref for Files {
	type Target = IndexMap<PathBuf, File>;

	fn deref(&self) -> &Self::Target { &self.items }
}

impl DerefMut for Files {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.items }
}

struct FilesSort {
	pub by:      SortBy,
	pub reverse: bool,
}

impl Default for FilesSort {
	fn default() -> Self { Self { by: MANAGER.sort_by, reverse: MANAGER.sort_reverse } }
}

#[derive(Debug)]
pub enum FilesOp {
	Read(PathBuf, IndexMap<PathBuf, File>),
	IOErr(PathBuf),
	Search(PathBuf, IndexMap<PathBuf, File>),
}

impl FilesOp {
	#[inline]
	pub fn path(&self) -> PathBuf {
		match self {
			Self::Read(path, _) => path,
			Self::IOErr(path) => path,
			Self::Search(path, _) => path,
		}
		.clone()
	}

	#[inline]
	pub fn read_empty(path: &Path) -> Self { Self::Read(path.to_path_buf(), IndexMap::new()) }

	#[inline]
	pub fn search_empty(path: &Path) -> Self { Self::Search(path.to_path_buf(), IndexMap::new()) }
}
