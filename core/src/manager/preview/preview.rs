use std::{path::{Path, PathBuf}, sync::atomic::Ordering};

use adaptor::Adaptor;
use config::MANAGER;
use shared::{MimeKind, PeekError};
use tokio::task::JoinHandle;

use super::{Provider, INCR};
use crate::emit;

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

		self.reset();
		if !self.same_path(path) {
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
	pub fn arrow(&mut self, step: isize, absolute: bool) -> bool {
		let old = self.skip;
		if absolute {
			self.skip = step.unsigned_abs();
		} else if let Some(kind) = self.lock.as_ref().map(|l| MimeKind::new(&l.mime)) {
			let size = Provider::step_size(kind, step.unsigned_abs());
			self.skip = if step < 0 { old.saturating_sub(size) } else { old + size };
		}

		self.skip != old
	}

	pub fn reset(&mut self) -> bool {
		self.handle.take().map(|h| h.abort());
		INCR.fetch_add(1, Ordering::Relaxed);
		Adaptor::image_hide(MANAGER.layout.preview_rect()).ok();

		let b = matches!(&self.lock, Some(l) if !l.is_image());
		self.lock = None;
		b
	}

	pub fn reset_image(&mut self) -> bool {
		self.handle.take().map(|h| h.abort());
		INCR.fetch_add(1, Ordering::Relaxed);
		Adaptor::image_hide(MANAGER.layout.preview_rect()).ok();

		if matches!(&self.lock, Some(l) if l.is_image()) {
			self.lock = None;
		}
		false
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
