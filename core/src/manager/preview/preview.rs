use std::{mem, path::{Path, PathBuf}, sync::atomic::Ordering};

use adaptor::Adaptor;
use config::MANAGER;
use shared::{MimeKind, PeekError};
use tokio::task::JoinHandle;

use super::{provider::INCR, Provider};
use crate::emit;

#[derive(Default)]
pub struct Preview {
	pub lock: Option<(PathBuf, String)>,
	pub data: PreviewData,
	skip:     usize,

	handle: Option<JoinHandle<()>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum PreviewData {
	#[default]
	None,
	Folder,
	Text(String),
	Image,
}

impl Preview {
	pub fn go(&mut self, path: &Path, mime: &str, show_image: bool) {
		let kind = MimeKind::new(mime);
		if !show_image && matches!(kind, MimeKind::Image | MimeKind::Video) {
			return;
		} else if self.same(path, mime) {
			return;
		} else {
			self.reset();
		}

		let (path, mime, skip) = (path.to_path_buf(), mime.to_owned(), self.skip);
		self.handle = Some(tokio::spawn(async move {
			let result = Provider::auto(kind, &path, skip).await;
			emit!(Preview(path, mime, result.unwrap_or_default()));
		}));
	}

	pub fn peek(&mut self, skip: usize) {
		let Some((path, kind, mime)) = self.lock.clone().map(|(p, m)| (p, MimeKind::new(&m), m)) else {
			return;
		};

		self.handle.take().map(|h| h.abort());
		INCR.fetch_add(1, Ordering::Relaxed);

		self.skip = skip;
		self.handle = Some(tokio::spawn(async move {
			match Provider::auto(kind, &path, skip).await {
				Ok(result) => {
					emit!(Preview(path, mime, result));
				}
				Err(PeekError::Exceed(max)) => {
					emit!(Peek(path, max));
				}
				_ => {}
			};
		}));
	}

	#[inline]
	pub fn peek_step(&mut self, step: isize) -> usize {
		let Some(kind) = self.lock.clone().map(|(_, m)| MimeKind::new(&m)) else {
			return 0;
		};

		Provider::step_size(kind, step.unsigned_abs())
	}

	pub fn reset(&mut self) -> bool {
		self.handle.take().map(|h| h.abort());
		INCR.fetch_add(1, Ordering::Relaxed);
		Adaptor::image_hide(MANAGER.layout.preview_rect()).ok();

		self.lock = None;
		self.skip = 0;
		!matches!(
			mem::replace(&mut self.data, PreviewData::None),
			PreviewData::None | PreviewData::Image
		)
	}

	pub fn reset_image(&mut self) -> bool {
		self.handle.take().map(|h| h.abort());
		INCR.fetch_add(1, Ordering::Relaxed);
		Adaptor::image_hide(MANAGER.layout.preview_rect()).ok();

		if self.data == PreviewData::Image {
			self.lock = None;
			self.data = PreviewData::None;
			self.skip = 0;
		}
		false
	}
}

impl Preview {
	#[inline]
	pub fn skip(&self) -> usize { self.skip }

	#[inline]
	pub fn same(&self, path: &Path, mime: &str) -> bool {
		self.lock.as_ref().map(|(p, m)| p == path && m == mime).unwrap_or(false)
	}

	#[inline]
	pub fn same_path(&self, path: &Path) -> bool {
		self.lock.as_ref().map(|(p, _)| p == path).unwrap_or(false)
	}
}
