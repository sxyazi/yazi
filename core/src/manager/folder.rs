use std::path::{Path, PathBuf};

use indexmap::map::Slice;
use ratatui::layout::Rect;
use shared::tty_size;

use super::{ALL_RATIO, CURRENT_RATIO, DIR_PADDING, PARENT_RATIO};
use crate::{emit, files::{File, Files, FilesOp}};

#[derive(Default)]
pub struct Folder {
	pub cwd:   PathBuf,
	pub files: Files,
	offset:    usize,
	cursor:    usize,

	pub page:      usize,
	pub hovered:   Option<File>,
	pub in_search: bool,
}

impl Folder {
	pub fn new(cwd: &Path) -> Self { Self { cwd: cwd.to_path_buf(), ..Default::default() } }

	pub fn new_search(cwd: &Path) -> Self {
		Self { cwd: cwd.to_path_buf(), in_search: true, ..Default::default() }
	}

	#[inline]
	pub fn limit() -> usize { tty_size().ws_row.saturating_sub(DIR_PADDING) as usize }

	pub fn update(&mut self, op: FilesOp) -> bool {
		let b = match op {
			FilesOp::Read(_, items) => self.files.update_read(items),
			FilesOp::Sort(_, items) => self.files.update_sort(items),
			FilesOp::Search(_, items) => self.files.update_search(items),
			_ => unreachable!(),
		};
		if !b {
			return false;
		}

		let len = self.files.len();
		self.offset = self.offset.min(len);
		self.cursor = self.cursor.min(len.saturating_sub(1));
		self.set_page(true);

		if let Some(h) = self.hovered.as_ref().map(|h| h.path()) {
			self.hover(&h);
		}
		self.hovered = self.files.duplicate(self.cursor);

		true
	}

	pub fn set_page(&mut self, force: bool) -> bool {
		let limit = Self::limit();
		let new = if limit == 0 { 0 } else { self.cursor / limit };
		if !force && self.page == new {
			return false;
		}

		self.page = new;
		emit!(Pages(new));
		true
	}

	pub fn next(&mut self, step: usize) -> bool {
		let len = self.files.len();
		if len == 0 {
			return false;
		}

		let old = self.cursor;
		self.cursor = (self.cursor + step).min(len - 1);
		self.hovered = self.files.duplicate(self.cursor);
		self.set_page(false);

		let limit = Self::limit();
		if self.cursor >= (self.offset + limit).min(len).saturating_sub(5) {
			self.offset = len.saturating_sub(limit).min(self.offset + self.cursor - old);
		}

		old != self.cursor
	}

	pub fn prev(&mut self, step: usize) -> bool {
		let old = self.cursor;
		self.cursor = self.cursor.saturating_sub(step);
		self.hovered = self.files.duplicate(self.cursor);
		self.set_page(false);

		if self.cursor < self.offset + 5 {
			self.offset = self.offset.saturating_sub(old - self.cursor);
		}

		old != self.cursor
	}

	pub fn hidden(&mut self, show: Option<bool>) -> bool {
		if show.is_none() || self.files.show_hidden != show.unwrap() {
			self.files.show_hidden = !self.files.show_hidden;
			emit!(Refresh);
		}

		false
	}

	#[inline]
	pub fn window(&self) -> &Slice<PathBuf, File> {
		let end = (self.offset + Self::limit()).min(self.files.len());
		self.files.get_range(self.offset..end).unwrap()
	}

	pub fn select(&mut self, idx: Option<usize>, state: Option<bool>) -> bool {
		let len = self.files.len();
		let mut apply = |idx: usize, state: Option<bool>| -> bool {
			let Some(state) = state else {
				self.files[idx].is_selected = !self.files[idx].is_selected;
				return true;
			};

			if state != self.files[idx].is_selected {
				self.files[idx].is_selected = state;
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

	pub fn hover(&mut self, path: &Path) -> bool {
		let new = self.position(path).unwrap_or(self.cursor);
		if new > self.cursor { self.next(new - self.cursor) } else { self.prev(self.cursor - new) }
	}

	pub fn hover_force(&mut self, file: File) -> bool {
		if self.hover(&file.path) {
			return true;
		}

		self.hovered = Some(file);
		false
	}
}

impl Folder {
	#[inline]
	pub fn cursor(&self) -> usize { self.cursor }

	#[inline]
	pub fn position(&self, path: &Path) -> Option<usize> {
		self.files.iter().position(|(p, _)| p == path)
	}

	pub fn paginate(&self) -> &Slice<PathBuf, File> {
		let len = self.files.len();
		let limit = Self::limit();

		let start = (self.page * limit).min(len.saturating_sub(1));
		let end = (start + limit).min(len);
		self.files.get_range(start..end).unwrap()
	}

	#[inline]
	pub fn has_selected(&self) -> bool { self.files.iter().any(|(_, f)| f.is_selected) }

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
