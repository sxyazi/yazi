use std::path::{Path, PathBuf};

use config::MANAGER;
use ratatui::layout::Rect;

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

	pub fn update(&mut self, op: FilesOp) -> bool {
		let b = match op {
			FilesOp::Read(_, items) => self.files.update_read(items),
			FilesOp::Size(_, items) => self.files.update_size(items),
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

		if let Some(h) = self.hovered.as_ref().map(|h| h.path_owned()) {
			self.hover(&h);
		}
		self.hovered = self.files.duplicate(self.cursor);

		true
	}

	pub fn set_page(&mut self, force: bool) -> bool {
		let limit = MANAGER.layout.folder_height();
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

		let limit = MANAGER.layout.folder_height();
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
		if self.files.set_show_hidden(show) {
			emit!(Refresh);
		}
		false
	}

	#[inline]
	pub fn window(&self) -> &[File] {
		let end = (self.offset + MANAGER.layout.folder_height()).min(self.files.len());
		&self.files[self.offset..end]
	}

	#[inline]
	pub fn window_for(&self, offset: usize) -> &[File] {
		let start = offset.min(self.files.len().saturating_sub(1));
		let end = (offset + MANAGER.layout.folder_height()).min(self.files.len());
		&self.files[start..end]
	}

	pub fn hover(&mut self, path: &Path) -> bool {
		let new = self.files.position(path).unwrap_or(self.cursor);
		if new > self.cursor { self.next(new - self.cursor) } else { self.prev(self.cursor - new) }
	}

	pub fn hover_force(&mut self, file: File) -> bool {
		if self.hover(file.path()) {
			return true;
		}

		self.hovered = Some(file);
		false
	}
}

impl Folder {
	#[inline]
	pub fn offset(&self) -> usize { self.offset }

	#[inline]
	pub fn cursor(&self) -> usize { self.cursor }

	pub fn paginate(&self) -> &[File] {
		let len = self.files.len();
		let limit = MANAGER.layout.folder_height();

		let start = (self.page * limit).min(len.saturating_sub(1));
		let end = (start + limit).min(len);
		&self.files[start..end]
	}

	pub fn rect_current(&self, path: &Path) -> Option<Rect> {
		let y = self.files.position(path)? - self.offset;

		let mut rect = MANAGER.layout.folder_rect();
		rect.y = rect.y.saturating_sub(1) + y as u16;
		rect.height = 1;
		Some(rect)
	}
}
