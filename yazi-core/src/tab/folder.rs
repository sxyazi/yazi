use std::mem;

use yazi_config::{LAYOUT, YAZI};
use yazi_dds::Pubsub;
use yazi_fs::{Entries, FilesOp, FolderStage, cha::Cha, file::File};
use yazi_macro::err;
use yazi_shared::{id::Id, path::{DynPath, PathBufDyn, PathDyn}, url::UrlBuf};
use yazi_widgets::{Scrollable, Step};

use crate::MgrProxy;

pub struct Folder {
	pub url:     UrlBuf,
	pub cha:     Cha,
	pub entries: Entries,
	pub stage:   FolderStage,

	pub offset: usize,
	pub cursor: usize,

	pub page:  usize,
	pub trace: Option<PathBufDyn>,
}

impl Default for Folder {
	fn default() -> Self {
		Self {
			url:     Default::default(),
			cha:     Default::default(),
			entries: Entries::new(YAZI.mgr.show_hidden.get()),
			stage:   Default::default(),
			offset:  Default::default(),
			cursor:  Default::default(),
			page:    Default::default(),
			trace:   Default::default(),
		}
	}
}

impl<T: Into<UrlBuf>> From<T> for Folder {
	fn from(value: T) -> Self { Self { url: value.into(), ..Default::default() } }
}

impl Folder {
	pub fn update(&mut self, op: FilesOp) -> bool {
		let (stage, revision) = (self.stage.clone(), self.entries.revision);
		match op {
			FilesOp::Full(_, _, cha) => {
				(self.cha, self.stage) = (cha, FolderStage::Loaded);
			}
			FilesOp::Part(_, ref files, _) if files.is_empty() => {
				(self.cha, self.stage) = (Cha::default(), FolderStage::Loading);
			}
			FilesOp::Part(_, _, ticket) if ticket == self.entries.ticket() => {
				self.stage = FolderStage::Loading;
			}
			FilesOp::Done(_, cha, ticket) if ticket == self.entries.ticket() => {
				(self.cha, self.stage) = (cha, FolderStage::Loaded);
			}
			FilesOp::IOErr(_, ref err) => {
				(self.cha, self.stage) = (Cha::default(), FolderStage::Failed(err.clone()));
			}
			_ => {}
		}

		let mut deleted = vec![];
		match op {
			FilesOp::Full(_, files, _) => self.entries.update_full(files),
			FilesOp::Part(_, files, ticket) => self.entries.update_part(files, ticket),
			FilesOp::Done(..) => {}
			FilesOp::Size(_, sizes) => self.entries.update_size(sizes),
			FilesOp::IOErr(..) => self.entries.update_ioerr(),

			FilesOp::Creating(_, files) => self.entries.update_creating(files),
			FilesOp::Deleting(_, urns) => deleted = self.entries.update_deleting(urns),
			FilesOp::Updating(_, files) => _ = self.entries.update_updating(files),
			FilesOp::Upserting(_, files) => self.entries.update_upserting(files),
		};

		self.trace.take_if(|_| self.entries.is_empty() && !self.stage.is_loading());
		self.arrow(-(deleted.into_iter().filter(|&i| i < self.cursor).count() as isize));
		self.repos(None);

		(&stage, revision) != (&self.stage, self.entries.revision)
	}

	pub fn update_pub(&mut self, tab: Id, op: FilesOp) -> bool {
		if self.update(op) {
			err!(Pubsub::pub_after_load(tab, &self.url, &self.stage));
			return true;
		}
		false
	}

	pub fn arrow(&mut self, step: impl Into<Step>) -> bool {
		let mut b = if self.entries.is_empty() {
			(mem::take(&mut self.cursor), mem::take(&mut self.offset)) != (0, 0)
		} else {
			self.scroll(step)
		};

		b |= self.squeeze_offset();
		self.sync_page(false);
		b
	}

	pub fn hover(&mut self, key: PathDyn) -> bool {
		if key.is_empty() {
			return self.arrow(0);
		} else if self.hovered().map(|h| h.entry_key()) == Some(key) {
			return self.arrow(0);
		}

		let new = self.entries.position(key).unwrap_or(self.cursor) as isize;
		let b = self.arrow(new - self.cursor as isize);

		self.retrace();
		b
	}

	pub fn repos(&mut self, key: Option<PathDyn>) -> bool {
		if let Some(k) = key {
			self.hover(k)
		} else if let Some(k) = self.trace.take() {
			let b = self.hover(k.dyn_path());
			self.trace = Some(k);
			b
		} else {
			self.arrow(0)
		}
	}

	pub fn retrace(&mut self) {
		self.trace = self.hovered().map(|h| h.entry_key().into()).or(self.trace.take());
	}

	pub fn sync_page(&mut self, force: bool) {
		let limit = LAYOUT.get().folder_limit();
		if limit == 0 {
			return;
		}

		let new = self.cursor / limit;
		if mem::replace(&mut self.page, new) != new || force {
			MgrProxy::update_paged_by(new, &self.url);
		}
	}

	fn squeeze_offset(&mut self) -> bool {
		let old = self.offset;
		let len = self.entries.len();

		let limit = LAYOUT.get().folder_limit();
		let scrolloff = (limit / 2).min(YAZI.mgr.scrolloff.get() as usize);

		self.offset = if self.cursor < (self.offset + limit).min(len).saturating_sub(scrolloff) {
			len.saturating_sub(limit).min(self.offset)
		} else {
			len.saturating_sub(limit).min(self.cursor.saturating_sub(limit) + 1 + scrolloff)
		}
		.min(self.cursor);

		old != self.offset
	}
}

impl Folder {
	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.entries.get(self.cursor) }

	#[inline]
	pub fn hovered_mut(&mut self) -> Option<&mut File> { self.entries.get_mut(self.cursor) }

	#[inline]
	pub fn hovered_url(&self) -> Option<&UrlBuf> { self.hovered().map(|f| &f.url) }

	pub fn paginate(&self, page: usize) -> &[File] {
		let len = self.entries.len();
		let limit = LAYOUT.get().folder_limit();

		let start = (page.saturating_sub(1) * limit).min(len.saturating_sub(1));
		let end = ((page + 2) * limit).min(len);
		&self.entries[start..end]
	}
}

impl Scrollable for Folder {
	fn total(&self) -> usize { self.entries.len() }

	fn limit(&self) -> usize { LAYOUT.get().folder_limit() }

	fn scrolloff(&self) -> usize { (self.limit() / 2).min(YAZI.mgr.scrolloff.get() as usize) }

	fn cursor_mut(&mut self) -> &mut usize { &mut self.cursor }

	fn offset_mut(&mut self) -> &mut usize { &mut self.offset }
}
