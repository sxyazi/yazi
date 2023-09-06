use std::{sync::atomic::Ordering, time::Duration};

use adaptor::Adaptor;
use config::MANAGER;
use shared::{MimeKind, PeekError, Url, MIME_DIR};
use tokio::{pin, task::JoinHandle};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

use super::{Provider, INCR};
use crate::{emit, files::{Files, FilesOp}};

#[derive(Default)]
pub struct Preview {
	pub lock: Option<PreviewLock>,
	skip:     usize,

	handle: Option<JoinHandle<()>>,
}

pub struct PreviewLock {
	pub url:  Url,
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
	pub fn go(&mut self, url: &Url, mime: &str, show_image: bool) {
		let kind = MimeKind::new(mime);
		if !show_image && kind.show_as_image() {
			return;
		} else if self.same(url, mime) {
			return;
		}

		self.reset(|_| true);
		if !self.same_mime(url, mime) {
			self.skip = 0;
		}

		let (url, mime, skip) = (url.clone(), mime.to_owned(), self.skip);
		self.handle = Some(tokio::spawn(async move {
			match Provider::auto(kind, &url, skip).await {
				Ok(data) => {
					emit!(Preview(PreviewLock { url, mime, skip, data }));
				}
				Err(PeekError::Exceed(max)) => {
					emit!(Peek(max, url));
				}
				_ => {}
			}
		}));
	}

	pub fn folder(&mut self, url: &Url, files: Option<usize>, sequent: bool) {
		if let Some(files) = files {
			self.skip = self.skip.min(files.saturating_sub(MANAGER.layout.preview_height()));
		}

		if self.same(url, MIME_DIR) {
			return;
		} else if !self.same_mime(url, MIME_DIR) {
			self.skip = 0;
		}

		self.reset(|_| true);
		emit!(Preview(PreviewLock {
			url:  url.clone(),
			mime: MIME_DIR.to_owned(),
			skip: self.skip,
			data: PreviewData::Folder,
		}));

		if sequent {
			return;
		}

		let url = url.clone();
		self.handle = Some(tokio::spawn(async move {
			let Ok(rx) = Files::from_dir(&url).await else {
				emit!(Files(FilesOp::IOErr(url)));
				return;
			};

			if files.is_some() {
				emit!(Files(FilesOp::Full(url, UnboundedReceiverStream::new(rx).collect().await)));
				return;
			}

			let rx = UnboundedReceiverStream::new(rx).chunks_timeout(10000, Duration::from_millis(500));
			pin!(rx);

			let version = FilesOp::prepare(&url);
			while let Some(chunk) = rx.next().await {
				emit!(Files(FilesOp::Part(url.clone(), version, chunk)));
			}
		}));
	}

	pub fn sequent(&mut self, url: &Url, mime: &str, show_image: bool) {
		let kind = MimeKind::new(mime);
		if !show_image && kind.show_as_image() {
			return;
		} else if self.same(url, mime) {
			return;
		}

		self.handle.take().map(|h| h.abort());
		INCR.fetch_add(1, Ordering::Relaxed);

		let (url, mime, skip) = (url.clone(), mime.to_owned(), self.skip);
		self.handle = Some(tokio::spawn(async move {
			match Provider::auto(kind, &url, skip).await {
				Ok(data) => {
					emit!(Preview(PreviewLock { url, mime, skip, data }));
				}
				Err(PeekError::Exceed(max)) => {
					emit!(Peek(max, url));
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
	pub fn same(&self, url: &Url, mime: &str) -> bool {
		if let Some(ref lock) = self.lock {
			return &lock.url == url && lock.mime == mime && lock.skip == self.skip;
		}
		false
	}

	#[inline]
	pub fn same_mime(&self, url: &Url, mime: &str) -> bool {
		if let Some(ref lock) = self.lock {
			return &lock.url == url && lock.mime == mime;
		}
		false
	}

	#[inline]
	pub fn same_path(&self, url: &Url) -> bool {
		if let Some(ref lock) = self.lock {
			return &lock.url == url;
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
