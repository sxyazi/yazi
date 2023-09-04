use std::{path::{Path, PathBuf}, sync::atomic::Ordering};

use adaptor::Adaptor;
use config::MANAGER;
use shared::{MimeKind, PeekError, MIME_DIR};
use tokio::task::JoinHandle;


use super::{Provider, INCR};
use crate::{emit, files::{Files, FilesOp}};

#[derive(Default)]
pub struct Preview {
	pub lock: Option<PreviewLock>,
	skip:     usize,

	handle: Option<JoinHandle<()>>,
}

pub struct PreviewLock {
	pub path: PathBuf,
	pub mime: String,
	pub skip: usize,
	pub data: PreviewData,
}

#[derive(Debug)]
pub enum PreviewData {
	Folder,
	Text(String),
	Image,
}

impl Preview {
	pub fn go(&mut self, path: &Path, mime: &str, show_image: bool) {
		let kind = MimeKind::new(mime);
		if !show_image && kind.show_as_image() {
			return;
		} else if self.same(path, mime) {
			return;
		}

		self.reset(|_| true);
		if !self.same_mime(path, mime) {
			self.skip = 0;
		}

		let (path, mime, skip) = (path.to_path_buf(), mime.to_owned(), self.skip);
		self.handle = Some(tokio::spawn(async move {
			match Provider::auto(kind, &path, skip).await {
				Ok(data) => {
					emit!(Preview(PreviewLock { path, mime, skip, data }));
				}
				Err(PeekError::Exceed(max)) => {
					emit!(Peek(max, path));
				}
				_ => {}
			}
		}));
	}

	pub fn folder(&mut self, path: &Path, files: Option<usize>, sequent: bool) {
		if let Some(files) = files {
			self.skip = self.skip.min(files.saturating_sub(MANAGER.layout.preview_height()));
		}

		if self.same(path, MIME_DIR) {
			return;
		} else if !self.same_mime(path, MIME_DIR) {
			self.skip = 0;
		}

		self.reset(|_| true);
		if files.is_some() || sequent {
			emit!(Preview(PreviewLock {
				path: path.to_path_buf(),
				mime: MIME_DIR.to_owned(),
				skip: self.skip,
				data: PreviewData::Folder,
			}));
		}

		if sequent {
			return;
		}

		let (path, skip) = (path.to_path_buf(), self.skip);
		self.handle = Some(tokio::spawn(async move {
			emit!(Files(match Files::read_dir(&path).await {
				Ok(items) => FilesOp::Read(path.clone(), items),
				Err(_) => FilesOp::IOErr(path.clone()),
			}));

			emit!(Preview(PreviewLock {
				path,
				mime: MIME_DIR.to_owned(),
				skip,
				data: PreviewData::Folder,
			}));
		}));
	}

	pub fn sequent(&mut self, path: &Path, mime: &str, show_image: bool) {
		let kind = MimeKind::new(mime);
		if !show_image && kind.show_as_image() {
			return;
		} else if self.same(path, mime) {
			return;
		}

		self.handle.take().map(|h| h.abort());
		INCR.fetch_add(1, Ordering::Relaxed);

		let (path, mime, skip) = (path.to_path_buf(), mime.to_owned(), self.skip);
		self.handle = Some(tokio::spawn(async move {
			match Provider::auto(kind, &path, skip).await {
				Ok(data) => {
					emit!(Preview(PreviewLock { path, mime, skip, data }));
				}
				Err(PeekError::Exceed(max)) => {
					emit!(Peek(max, path));
				}
				_ => {}
			}
		}));
	}

	#[inline]
	pub fn arrow(&mut self, step: isize) -> bool {
		let Some(kind) = self.lock.as_ref().map(|l| MimeKind::new(&l.mime)) else {
			return false;
		};

		let old = self.skip;
		let size = Provider::step_size(kind, step.unsigned_abs());

		self.skip = if step < 0 { old.saturating_sub(size) } else { old + size };
		self.skip != old
	}

	pub fn reset<F: FnOnce(&PreviewLock) -> bool>(&mut self, f: F) -> bool {
		self.handle.take().map(|h| h.abort());
		INCR.fetch_add(1, Ordering::Relaxed);
		Adaptor::image_hide(MANAGER.layout.preview_rect()).ok();

		let Some(ref lock) = self.lock else {
			return false;
		};

		let b = !lock.is_image();
		if f(lock) {
			self.lock = None;
		}
		b
	}
}

impl Preview {
	#[inline]
	pub fn lock(&self) -> &Option<PreviewLock> { &self.lock }

	#[inline]
	pub fn skip(&self) -> usize { self.skip }

	#[inline]
	pub fn same(&self, path: &Path, mime: &str) -> bool {
		if let Some(ref lock) = self.lock {
			return lock.path == path && lock.mime == mime && lock.skip == self.skip;
		}
		false
	}

	#[inline]
	pub fn same_mime(&self, path: &Path, mime: &str) -> bool {
		if let Some(ref lock) = self.lock {
			return lock.path == path && lock.mime == mime;
		}
		false
	}

	#[inline]
	pub fn same_path(&self, path: &Path) -> bool {
		if let Some(ref lock) = self.lock {
			return lock.path == path;
		}
		false
	}
}

impl PreviewLock {
	#[inline]
	pub fn is_image(&self) -> bool { matches!(self.data, PreviewData::Image) }

	#[inline]
	pub fn is_folder(&self) -> bool { matches!(self.data, PreviewData::Folder) }
}
