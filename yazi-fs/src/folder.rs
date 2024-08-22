use std::mem;

use yazi_config::{LAYOUT, MANAGER};
use yazi_proxy::ManagerProxy;
use yazi_shared::fs::{Cha, File, FilesOp, Url};

use super::FolderStage;
use crate::{Files, Step};

#[derive(Default)]
pub struct Folder {
	pub cwd:   Url,
	pub cha:   Cha,
	pub files: Files,
	pub stage: FolderStage,

	pub offset: usize,
	pub cursor: usize,

	pub page:    usize,
	pub tracing: bool,
}

impl From<&Url> for Folder {
	fn from(cwd: &Url) -> Self { Self { cwd: cwd.clone(), ..Default::default() } }
}

impl Folder {
	pub fn update(&mut self, op: FilesOp) -> bool {
		let (stage, revision) = (self.stage, self.files.revision);
		match op {
			FilesOp::Full(_, _, cha) => {
				(self.cha, self.stage) = (cha, FolderStage::Loaded);
			}
			FilesOp::Part(_, _, ticket) if ticket == self.files.ticket() => {
				self.stage = FolderStage::Loading;
			}
			FilesOp::Done(_, cha, ticket) if ticket == self.files.ticket() => {
				(self.cha, self.stage) = (cha, FolderStage::Loaded);
			}
			FilesOp::IOErr(_, kind) => {
				(self.cha, self.stage) = (Cha::dummy(), FolderStage::Failed(kind));
			}
			_ => {}
		}

		match op {
			FilesOp::Full(_, files, _) => self.files.update_full(files),
			FilesOp::Part(_, files, ticket) => self.files.update_part(files, ticket),
			FilesOp::Done(..) => {}
			FilesOp::Size(_, sizes) => self.files.update_size(sizes),
			FilesOp::IOErr(..) => self.files.update_ioerr(),

			FilesOp::Creating(_, files) => self.files.update_creating(files),
			FilesOp::Deleting(_, urls) => self.files.update_deleting(urls),
			FilesOp::Updating(_, files) => _ = self.files.update_updating(files),
			FilesOp::Upserting(_, files) => self.files.update_upserting(files),
		}

		self.arrow(0);
		(stage, revision) != (self.stage, self.files.revision)
	}

	pub fn arrow(&mut self, step: impl Into<Step>) -> bool {
		let step = step.into() as Step;
		let b = if self.files.is_empty() {
			(self.cursor, self.offset, self.tracing) = (0, 0, false);
			false
		} else if step.is_positive() {
			self.next(step)
		} else {
			self.prev(step)
		};

		self.sync_page(false);
		self.tracing |= b;
		b
	}

	pub fn hover(&mut self, url: &Url) -> bool {
		if self.hovered().map(|h| &h.url) == Some(url) {
			return false;
		}

		let new = self.files.position(url).unwrap_or(self.cursor) as isize;
		self.arrow(new - self.cursor as isize)
	}

	#[inline]
	pub fn repos(&mut self, url: Option<impl AsRef<Url>>) -> bool {
		if let Some(u) = url { self.hover(u.as_ref()) } else { self.arrow(0) }
	}

	pub fn sync_page(&mut self, force: bool) {
		let limit = LAYOUT.load().current.height as usize;
		if limit == 0 {
			return;
		}

		let new = self.cursor / limit;
		if mem::replace(&mut self.page, new) != new || force {
			ManagerProxy::update_paged_by(new, &self.cwd);
		}
	}

	fn next(&mut self, step: Step) -> bool {
		let old = (self.cursor, self.offset);
		let len = self.files.len();

		let limit = LAYOUT.load().current.height as usize;
		let scrolloff = (limit / 2).min(MANAGER.scrolloff as usize);

		self.cursor = step.add(self.cursor, limit).min(len.saturating_sub(1));
		self.offset = if self.cursor >= (self.offset + limit).min(len).saturating_sub(scrolloff) {
			len.saturating_sub(limit).min(self.offset + self.cursor - old.0)
		} else {
			self.offset.min(len.saturating_sub(1))
		};

		old != (self.cursor, self.offset)
	}

	fn prev(&mut self, step: Step) -> bool {
		let old = (self.cursor, self.offset);
		let max = self.files.len().saturating_sub(1);

		let limit = LAYOUT.load().current.height as usize;
		let scrolloff = (limit / 2).min(MANAGER.scrolloff as usize);

		self.cursor = step.add(self.cursor, limit).min(max);
		self.offset = if self.cursor < self.offset + scrolloff {
			self.offset.saturating_sub(old.0 - self.cursor)
		} else {
			self.offset.min(max)
		};

		old != (self.cursor, self.offset)
	}
}

impl Folder {
	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.files.get(self.cursor) }

	pub fn paginate(&self, page: usize) -> &[File] {
		let len = self.files.len();
		let limit = LAYOUT.load().current.height as usize;

		let start = (page.saturating_sub(1) * limit).min(len.saturating_sub(1));
		let end = ((page + 2) * limit).min(len);
		&self.files[start..end]
	}
}
