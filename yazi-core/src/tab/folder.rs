use std::mem;

use yazi_config::{LAYOUT, YAZI};
use yazi_dds::Pubsub;
use yazi_fs::{File, Files, FilesOp, FolderStage, cha::Cha};
use yazi_macro::err;
use yazi_parser::Step;
use yazi_proxy::MgrProxy;
use yazi_shared::{Id, path::{AsPath, PathBufDyn, PathDyn}, url::UrlBuf};
use yazi_widgets::Scrollable;

pub struct Folder {
	pub url:   UrlBuf,
	pub cha:   Cha,
	pub files: Files,
	pub stage: FolderStage,

	pub offset: usize,
	pub cursor: usize,

	pub page:  usize,
	pub trace: Option<PathBufDyn>,
}

impl Default for Folder {
	fn default() -> Self {
		Self {
			url:    Default::default(),
			cha:    Default::default(),
			files:  Files::new(YAZI.mgr.show_hidden.get()),
			stage:  Default::default(),
			offset: Default::default(),
			cursor: Default::default(),
			page:   Default::default(),
			trace:  Default::default(),
		}
	}
}

impl<T: Into<UrlBuf>> From<T> for Folder {
	fn from(value: T) -> Self { Self { url: value.into(), ..Default::default() } }
}

impl Folder {
	pub fn update(&mut self, op: FilesOp) -> bool {
		let (stage, revision) = (self.stage.clone(), self.files.revision);
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
			FilesOp::IOErr(_, ref err) => {
				(self.cha, self.stage) = (Cha::default(), FolderStage::Failed(err.clone()));
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
		self.repos(None);

		(&stage, revision) != (&self.stage, self.files.revision)
	}

	pub fn update_pub(&mut self, tab: Id, op: FilesOp) -> bool {
		if self.update(op) {
			err!(Pubsub::pub_after_load(tab, &self.url, &self.stage));
			return true;
		}
		false
	}

	pub fn arrow(&mut self, step: impl Into<Step>) -> bool {
		let mut b = if self.files.is_empty() {
			(mem::take(&mut self.cursor), mem::take(&mut self.offset)) != (0, 0)
		} else {
			self.scroll(step)
		};

		self.trace = self.hovered().filter(|_| b).map(|h| h.urn().into()).or(self.trace.take());
		b |= self.squeeze_offset();

		self.sync_page(false);
		b
	}

	pub fn hover(&mut self, urn: PathDyn) -> bool {
		if self.hovered().map(|h| h.urn()) == Some(urn) {
			return self.arrow(0);
		}

		let new = self.files.position(urn).unwrap_or(self.cursor) as isize;
		self.arrow(new - self.cursor as isize)
	}

	pub fn repos(&mut self, urn: Option<PathDyn>) -> bool {
		if let Some(u) = urn {
			self.hover(u)
		} else if let Some(u) = &self.trace {
			self.hover(u.clone().as_path())
		} else {
			self.arrow(0)
		}
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
		let len = self.files.len();

		let limit = LAYOUT.get().folder_limit();
		let scrolloff = (limit / 2).min(YAZI.mgr.scrolloff.get() as usize);

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

	#[inline]
	pub fn hovered_mut(&mut self) -> Option<&mut File> { self.files.get_mut(self.cursor) }

	pub fn paginate(&self, page: usize) -> &[File] {
		let len = self.files.len();
		let limit = LAYOUT.get().folder_limit();

		let start = (page.saturating_sub(1) * limit).min(len.saturating_sub(1));
		let end = ((page + 2) * limit).min(len);
		&self.files[start..end]
	}
}

impl Scrollable for Folder {
	fn total(&self) -> usize { self.files.len() }

	fn limit(&self) -> usize { LAYOUT.get().folder_limit() }

	fn scrolloff(&self) -> usize { (self.limit() / 2).min(YAZI.mgr.scrolloff.get() as usize) }

	fn cursor_mut(&mut self) -> &mut usize { &mut self.cursor }

	fn offset_mut(&mut self) -> &mut usize { &mut self.offset }
}
