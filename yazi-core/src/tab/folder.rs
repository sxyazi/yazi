use std::mem;

use yazi_config::{LAYOUT, YAZI};
use yazi_dds::Pubsub;
use yazi_fs::{File, Files, FilesOp, FolderStage, Step, cha::Cha};
use yazi_proxy::MgrProxy;
use yazi_shared::{Id, url::{Url, Urn, UrnBuf}};

pub struct Folder {
	pub url:   Url,
	pub cha:   Cha,
	pub files: Files,
	pub stage: FolderStage,

	pub offset: usize,
	pub cursor: usize,

	pub page:  usize,
	pub trace: Option<UrnBuf>,
}

impl Default for Folder {
	fn default() -> Self {
		Self {
			url:    Default::default(),
			cha:    Default::default(),
			files:  Files::new(YAZI.mgr.show_hidden),
			stage:  Default::default(),
			offset: Default::default(),
			cursor: Default::default(),
			page:   Default::default(),
			trace:  Default::default(),
		}
	}
}

impl From<&Url> for Folder {
	fn from(url: &Url) -> Self { Self { url: url.clone(), ..Default::default() } }
}

impl Folder {
	pub fn update(&mut self, op: FilesOp) -> bool {
		let (stage, revision) = (self.stage, self.files.revision);
		match op {
			FilesOp::Full(_, _, cha) => {
				(self.cha, self.stage) = (cha, FolderStage::Loaded);
			}
			FilesOp::Part(_, ref files, _) if files.is_empty() => {
				(self.cha, self.stage) = (Cha::default(), FolderStage::Loading);
			}
			FilesOp::Part(_, _, ticket) if ticket == self.files.ticket() => {
				self.stage = FolderStage::Loading;
			}
			FilesOp::Done(_, cha, ticket) if ticket == self.files.ticket() => {
				(self.cha, self.stage) = (cha, FolderStage::Loaded);
			}
			FilesOp::IOErr(_, kind) => {
				(self.cha, self.stage) = (Cha::default(), FolderStage::Failed(kind));
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
			FilesOp::Deleting(_, urns) => {
				let deleted = self.files.update_deleting(urns);
				let delta = deleted.into_iter().filter(|&i| i < self.cursor).count() as isize;
				self.arrow(-delta);
			}
			FilesOp::Updating(_, files) => _ = self.files.update_updating(files),
			FilesOp::Upserting(_, files) => self.files.update_upserting(files),
		}

		self.trace = self.trace.take_if(|_| !self.files.is_empty() || self.stage.is_loading());
		self.repos(self.trace.clone());

		(stage, revision) != (self.stage, self.files.revision)
	}

	pub fn update_pub(&mut self, tab: Id, op: FilesOp) -> bool {
		let old = self.stage;
		if !self.update(op) {
			return false;
		} else if self.stage != old {
			Pubsub::pub_from_load(tab, &self.url, self.stage);
		}
		true
	}

	pub fn arrow(&mut self, step: impl Into<Step>) -> bool {
		let new = (step.into() as Step).add(self.cursor, self.files.len(), LAYOUT.get().limit());
		let mut b = if self.files.is_empty() {
			(mem::take(&mut self.cursor), mem::take(&mut self.offset)) != (0, 0)
		} else if new > self.cursor {
			self.next(new)
		} else {
			self.prev(new)
		};

		self.trace = self.hovered().filter(|_| b).map(|h| h.urn_owned()).or(self.trace.take());
		b |= self.squeeze_offset();

		self.sync_page(false);
		b
	}

	pub fn hover(&mut self, urn: &Urn) -> bool {
		if self.hovered().map(|h| h.urn()) == Some(urn) {
			return self.arrow(0);
		}

		let new = self.files.position(urn).unwrap_or(self.cursor) as isize;
		self.arrow(new - self.cursor as isize)
	}

	#[inline]
	pub fn repos(&mut self, urn: Option<impl AsRef<Urn>>) -> bool {
		if let Some(u) = urn { self.hover(u.as_ref()) } else { self.arrow(0) }
	}

	pub fn sync_page(&mut self, force: bool) {
		let limit = LAYOUT.get().limit();
		if limit == 0 {
			return;
		}

		let new = self.cursor / limit;
		if mem::replace(&mut self.page, new) != new || force {
			MgrProxy::update_paged_by(new, &self.url);
		}
	}

	fn next(&mut self, new: usize) -> bool {
		let old = (self.cursor, self.offset);
		let len = self.files.len();

		let limit = LAYOUT.get().limit();
		let scrolloff = (limit / 2).min(YAZI.mgr.scrolloff as usize);

		self.cursor = new;
		self.offset = if self.cursor < (self.offset + limit).min(len).saturating_sub(scrolloff) {
			self.offset.min(len.saturating_sub(1))
		} else {
			len.saturating_sub(limit).min(self.offset + self.cursor - old.0)
		};

		old != (self.cursor, self.offset)
	}

	fn prev(&mut self, new: usize) -> bool {
		let old = (self.cursor, self.offset);

		let limit = LAYOUT.get().limit();
		let scrolloff = (limit / 2).min(YAZI.mgr.scrolloff as usize);

		self.cursor = new;
		self.offset = if self.cursor < self.offset + scrolloff {
			self.offset.saturating_sub(old.0 - self.cursor)
		} else {
			self.offset.min(self.files.len().saturating_sub(1))
		};

		old != (self.cursor, self.offset)
	}

	fn squeeze_offset(&mut self) -> bool {
		let old = self.offset;
		let len = self.files.len();

		let limit = LAYOUT.get().limit();
		let scrolloff = (limit / 2).min(YAZI.mgr.scrolloff as usize);

		self.offset = if self.cursor < (self.offset + limit).min(len).saturating_sub(scrolloff) {
			len.saturating_sub(limit).min(self.offset)
		} else {
			len.saturating_sub(limit).min(self.cursor.saturating_sub(limit) + 1 + scrolloff)
		};

		old != self.offset
	}
}

impl Folder {
	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.files.get(self.cursor) }

	pub fn paginate(&self, page: usize) -> &[File] {
		let len = self.files.len();
		let limit = LAYOUT.get().limit();

		let start = (page.saturating_sub(1) * limit).min(len.saturating_sub(1));
		let end = ((page + 2) * limit).min(len);
		&self.files[start..end]
	}
}
