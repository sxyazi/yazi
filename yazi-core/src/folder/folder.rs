use ratatui::layout::Rect;
use yazi_config::LAYOUT;
use yazi_shared::{emit, fs::{File, FilesOp, Url}};

use crate::{folder::Files, Step};

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
		let revision = self.files.revision;
		match op {
			FilesOp::Full(_, files) => self.files.update_full(files),
			FilesOp::Part(_, files, ticket) => self.files.update_part(files, ticket),
			FilesOp::Size(_, sizes) => self.files.update_size(sizes),

			FilesOp::Creating(_, files) => self.files.update_creating(files),
			FilesOp::Deleting(_, urls) => self.files.update_deleting(urls),
			FilesOp::Updating(_, files) => _ = self.files.update_updating(files),
			FilesOp::Upserting(_, files) => self.files.update_upserting(files),
			_ => unreachable!(),
		}
		if revision == self.files.revision {
			return false;
		}

		let old = self.page;
		self.prev(Default::default());

		if self.page == old {
			self.set_page(true); // Force update
		}

		true
	}

	pub fn set_page(&mut self, force: bool) {
		let limit = LAYOUT.load().current.height as usize;
		if limit == 0 {
			return;
		}

		let new = self.cursor / limit;
		if !force && self.page == new {
			return;
		}

		// Current page
		emit!(Pages(new));

		// Next page
		let max_page = (self.files.len() + limit - 1) / limit;
		if new < max_page && new + 1 != self.page {
			emit!(Pages(new + 1));
		}

		// Previous page
		if new > 1 && new - 1 != self.page {
			emit!(Pages(new - 1));
		}

		self.page = new;
	}

	pub fn next(&mut self, step: Step) -> bool {
		let old = (self.cursor, self.offset);
		let len = self.files.len();

		let limit = LAYOUT.load().current.height as usize;
		self.cursor = step.add(self.cursor, limit).min(len.saturating_sub(1));
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

		self.cursor = step.add(self.cursor, LAYOUT.load().current.height as usize).min(max);
		self.offset = if self.cursor < self.offset + 5 {
			self.offset.saturating_sub(old.0 - self.cursor)
		} else {
			self.offset.min(max)
		};

		self.set_page(false);
		old != (self.cursor, self.offset)
	}

	pub fn hover(&mut self, url: &Url) -> bool {
		if self.hovered().map(|h| &h.url) == Some(url) {
			return false;
		}

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

	pub fn paginate(&self, page: usize) -> &[File] {
		let len = self.files.len();
		let limit = LAYOUT.load().current.height as usize;

		let start = (page * limit).min(len.saturating_sub(1));
		let end = (start + limit).min(len);
		&self.files[start..end]
	}

	pub fn rect_current(&self, url: &Url) -> Option<Rect> {
		let y = self.files.position(url)? - self.offset;

		let mut rect = LAYOUT.load().current;
		rect.y = rect.y.saturating_sub(1) + y as u16;
		rect.height = 1;
		Some(rect)
	}
}
