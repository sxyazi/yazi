use yazi_config::MANAGER;
use ratatui::layout::Rect;
use yazi_shared::Url;

use crate::{emit, files::{File, Files, FilesOp}, Step};

#[derive(Default)]
pub struct Folder {
	pub cwd:   Url,
	pub files: Files,

	pub offset: usize,
	pub cursor: usize,

	pub page: usize,
}

impl From<Url> for Folder {
	fn from(cwd: Url) -> Self { Self { cwd, ..Default::default() } }
}

impl From<&Url> for Folder {
	fn from(cwd: &Url) -> Self { Self::from(cwd.clone()) }
}

impl Folder {
	pub fn update(&mut self, op: FilesOp) -> bool {
		let b = match op {
			FilesOp::Full(_, items) => self.files.update_full(items),
			FilesOp::Part(_, ticket, items) => self.files.update_part(ticket, items),
			FilesOp::Size(_, items) => self.files.update_size(items),

			FilesOp::Creating(_, items) => self.files.update_creating(items),
			FilesOp::Deleting(_, items) => self.files.update_deleting(items),
			FilesOp::Replacing(_, mut items) => self.files.update_replacing(&mut items),
			_ => unreachable!(),
		};
		if !b {
			return false;
		}

		let old = self.page;
		self.prev(Default::default());

		if self.page == old {
			emit!(Pages(self.page)); // Force update
		}

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

	pub fn next(&mut self, step: Step) -> bool {
		let old = (self.cursor, self.offset);
		let len = self.files.len();

		let limit = MANAGER.layout.folder_height();
		self.cursor = step.add(self.cursor, || limit).min(len.saturating_sub(1));
		self.offset = if self.cursor >= (self.offset + limit).min(len).saturating_sub(5) {
			len.saturating_sub(limit).min(self.offset + self.cursor - old.0)
		} else {
			self.offset.min(len.saturating_sub(1))
		};

		self.set_page(false);
		old != (self.cursor, self.offset)
	}

	pub fn prev(&mut self, step: Step) -> bool {
		let old = (self.cursor, self.offset);
		let max = self.files.len().saturating_sub(1);

		self.cursor = step.add(self.cursor, || MANAGER.layout.folder_height()).min(max);
		self.offset = if self.cursor < self.offset + 5 {
			self.offset.saturating_sub(old.0 - self.cursor)
		} else {
			self.offset.min(max)
		};

		self.set_page(false);
		old != (self.cursor, self.offset)
	}

	pub fn hover(&mut self, url: &Url) -> bool {
		let new = self.files.position(url).unwrap_or(self.cursor);
		if new > self.cursor {
			self.next(Step::next(new - self.cursor))
		} else {
			self.prev(Step::prev(self.cursor - new))
		}
	}

	#[inline]
	pub fn repos(&mut self, url: Option<impl AsRef<Url>>) -> bool {
		if let Some(u) = url { self.hover(u.as_ref()) } else { self.prev(Default::default()) }
	}
}

impl Folder {
	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.files.get(self.cursor) }

	pub fn paginate(&self) -> &[File] {
		let len = self.files.len();
		let limit = MANAGER.layout.folder_height();

		let start = (self.page * limit).min(len.saturating_sub(1));
		let end = (start + limit).min(len);
		&self.files[start..end]
	}

	pub fn rect_current(&self, url: &Url) -> Option<Rect> {
		let y = self.files.position(url)? - self.offset;

		let mut rect = MANAGER.layout.folder_rect();
		rect.y = rect.y.saturating_sub(1) + y as u16;
		rect.height = 1;
		Some(rect)
	}
}
