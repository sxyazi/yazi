use std::{fs::Metadata, path::{Path, PathBuf}, usize};

use indexmap::{map::Slice, IndexMap};
use ratatui::layout::Rect;
use tokio::fs;

use super::{ALL_RATIO, CURRENT_RATIO, DIR_PADDING, PARENT_RATIO};
use crate::{config::{manager::SortBy, MANAGER}, emit, misc::tty_size};

#[derive(Default)]
pub struct Folder {
	pub cwd: PathBuf,
	items:   IndexMap<PathBuf, FolderItem>,
	offset:  usize,
	cursor:  usize,

	sort:        FolderSort,
	show_hidden: bool,
}

#[derive(Clone)]
pub struct FolderItem {
	pub name:        String,
	pub path:        PathBuf,
	pub meta:        Metadata,
	pub length:      Option<u64>,
	pub is_link:     bool,
	pub is_hidden:   bool,
	pub is_selected: bool,
}

struct FolderSort {
	pub by:      SortBy,
	pub reverse: bool,
}

impl Default for FolderSort {
	fn default() -> Self { Self { by: MANAGER.sort_by, reverse: MANAGER.sort_reverse } }
}

impl Folder {
	pub fn new(cwd: &Path) -> Self {
		Self { cwd: cwd.to_path_buf(), show_hidden: MANAGER.show_hidden, ..Default::default() }
	}

	#[inline]
	pub fn limit() -> usize { tty_size().ws_row.saturating_sub(DIR_PADDING) as usize }

	pub async fn read(path: &Path) {
		let mut iter = match fs::read_dir(path).await {
			Ok(it) => it,
			Err(_) => return,
		};

		let mut items = IndexMap::new();
		while let Ok(Some(item)) = iter.next_entry().await {
			let mut meta = if let Ok(meta) = item.metadata().await { meta } else { continue };
			let is_link = meta.is_symlink();
			if is_link {
				meta = fs::metadata(&path).await.unwrap_or(meta);
			}

			let path = item.path();
			let name = item.file_name().to_string_lossy().to_string();

			let length = if meta.is_dir() { None } else { Some(meta.len()) };
			let is_hidden = name.starts_with('.');

			items.insert(path.clone(), FolderItem {
				name,
				path,
				meta,
				length,
				is_link,
				is_hidden,
				is_selected: false,
			});
		}
		emit!(Files(path.to_path_buf(), items));
	}

	pub fn sort(&mut self) {
		fn cmp<T: Ord>(a: T, b: T, reverse: bool) -> std::cmp::Ordering {
			if reverse { b.cmp(&a) } else { a.cmp(&b) }
		}

		let reverse = self.sort.reverse;
		match self.sort.by {
			SortBy::Alphabetical => self.items.sort_by(|_, a, _, b| cmp(&a.name, &b.name, reverse)),
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

	pub fn update(&mut self, mut items: IndexMap<PathBuf, FolderItem>) -> bool {
		if !self.show_hidden {
			items.retain(|_, item| !item.is_hidden);
		}

		for (path, item) in &mut items {
			if let Some(old) = self.items.get(path) {
				item.length = old.length;
				item.is_selected = old.is_selected;
			}
		}

		let len = items.len();
		self.items = items;
		self.cursor = self.cursor.min(len.saturating_sub(1));
		self.offset = self.offset.min(len);
		self.sort();
		true
	}

	pub fn next(&mut self, step: usize) -> bool {
		let len = self.items.len();
		if len == 0 {
			return false;
		}

		let old = self.cursor;
		self.cursor = (self.cursor + step).min(len - 1);

		let limit = Self::limit();
		if self.cursor >= (self.offset + limit).min(len).saturating_sub(5) {
			self.offset = len.saturating_sub(limit).min(self.offset + self.cursor - old);
		}

		old != self.cursor
	}

	pub fn prev(&mut self, step: usize) -> bool {
		let old = self.cursor;
		self.cursor = self.cursor.saturating_sub(step);

		if self.cursor < self.offset + 5 {
			self.offset = self.offset.saturating_sub(old - self.cursor);
		}

		old != self.cursor
	}

	pub fn hidden(&mut self, show: Option<bool>) -> bool {
		if show.is_none() || self.show_hidden != show.unwrap() {
			self.show_hidden = !self.show_hidden;
			emit!(Refresh);
		}

		false
	}

	pub fn paginate(&self) -> &Slice<PathBuf, FolderItem> {
		let end = (self.offset + Self::limit()).min(self.items.len());
		self.items.get_range(self.offset..end).unwrap()
	}

	pub fn select(&mut self, idx: Option<usize>, state: Option<bool>) -> bool {
		let len = self.items.len();
		let mut apply = |idx: usize, state: Option<bool>| -> bool {
			if state.is_none() {
				self.items[idx].is_selected = !self.items[idx].is_selected;
				return true;
			}

			let state = state.unwrap();
			if state != self.items[idx].is_selected {
				self.items[idx].is_selected = state;
				return true;
			}

			false
		};

		if let Some(idx) = idx {
			if idx < len {
				return apply(idx, state);
			}
		} else {
			let mut applied = false;
			for i in 0..len {
				if apply(i, state) {
					applied = true;
				}
			}
			return applied;
		}

		false
	}

	pub fn selected(&self) -> Option<Vec<PathBuf>> {
		let v = self
			.items
			.iter()
			.filter(|(_, item)| item.is_selected)
			.map(|(path, _)| path.clone())
			.collect::<Vec<_>>();

		if v.is_empty() { None } else { Some(v) }
	}

	pub fn hover(&mut self, path: &Path) -> bool {
		if self.hovered().map(|h| h.path.as_path()) == Some(path) {
			return false;
		}

		let new = self.position(path).unwrap_or(self.cursor);
		if new > self.cursor { self.next(new - self.cursor) } else { self.prev(self.cursor - new) }
	}
}

impl Folder {
	#[inline]
	pub fn hovered(&self) -> Option<&FolderItem> {
		self.items.get_index(self.cursor).map(|(_, item)| item)
	}

	#[inline]
	pub fn cursor(&self) -> usize { self.cursor }

	#[inline]
	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }

	#[inline]
	pub fn position(&self, path: &Path) -> Option<usize> {
		self.items.iter().position(|(p, _)| p == path)
	}

	#[inline]
	pub fn rect_current(&self, path: &Path) -> Option<Rect> {
		let pos = self.position(path)? - self.offset;
		let s = tty_size();

		Some(Rect {
			x:      (s.ws_col as u32 * PARENT_RATIO / ALL_RATIO) as u16,
			y:      pos as u16,
			width:  (s.ws_col as u32 * CURRENT_RATIO / ALL_RATIO) as u16,
			height: 1,
		})
	}
}
