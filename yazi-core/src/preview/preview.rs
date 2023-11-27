use std::{mem, time::Duration};

use tokio::{pin, task::JoinHandle};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_adaptor::ADAPTOR;
use yazi_config::MANAGER;
use yazi_shared::{fs::{Cha, Url}, MimeKind, PeekError};

use super::Provider;
use crate::{emit, files::{Files, FilesOp}, manager::Manager, Highlighter};

#[derive(Default)]
pub struct Preview {
	pub lock: Option<PreviewLock>,
	skip:     usize,

	handle: Option<JoinHandle<()>>,
}

pub struct PreviewLock {
	pub url:  Url,
	pub cha:  Option<Cha>,
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
	pub fn go(&mut self, url: &Url, cha: Cha, mime: &str) {
		if self.content_unchanged(url, &cha) {
			return;
		}

		self.abort();
		let (url, kind, skip) = (url.clone(), MimeKind::new(mime), self.skip);

		self.handle = Some(tokio::spawn(async move {
			match Provider::auto(kind, &url, skip).await {
				Ok(data) => {
					emit!(Preview(PreviewLock { url, cha: Some(cha), skip, data }));
				}
				Err(PeekError::Exceed(max)) => {
					Manager::_peek_upper_bound(max, &url);
				}
				_ => {}
			}
		}));
	}

	pub fn go_folder(&mut self, url: Url, in_chunks: bool) {
		self.abort();
		self.lock = Some(PreviewLock {
			url:  url.clone(),
			cha:  None,
			skip: self.skip,
			data: PreviewData::Folder,
		});

		self.handle = Some(tokio::spawn(async move {
			let Ok(rx) = Files::from_dir(&url).await else {
				emit!(Files(FilesOp::IOErr(url.clone())));
				return;
			};

			if !in_chunks {
				emit!(Files(FilesOp::Full(url.clone(), UnboundedReceiverStream::new(rx).collect().await)));
				return;
			}

			let stream =
				UnboundedReceiverStream::new(rx).chunks_timeout(10000, Duration::from_millis(500));
			pin!(stream);

			let ticket = FilesOp::prepare(&url);
			while let Some(chunk) = stream.next().await {
				emit!(Files(FilesOp::Part(url.clone(), ticket, chunk)));
			}
		}));
	}

	pub fn arrow(&mut self, step: isize, mime: &str, upper: Option<usize>) {
		let size = Provider::step_size(MimeKind::new(mime), step.unsigned_abs());
		self.skip = if step < 0 { self.skip.saturating_sub(size) } else { self.skip + size };

		if let Some(upper) = upper {
			self.skip = self.skip.min(upper);
		}
	}

	#[inline]
	pub fn abort(&mut self) {
		self.handle.take().map(|h| h.abort());
		Highlighter::abort();
		ADAPTOR.image_hide(MANAGER.layout.image_rect()).ok();
	}

	#[inline]
	pub fn reset(&mut self) -> bool {
		self.abort();
		self.lock.take().map(|l| l.is_image()) == Some(false)
	}

	pub fn reset_image(&mut self) -> bool {
		if matches!(self.lock, Some(ref l) if l.is_image()) {
			self.reset();
			true
		} else {
			false
		}
	}

	pub fn same_url(&self, url: &Url) -> bool {
		matches!(self.lock, Some(ref lock) if lock.url == *url)
	}

	pub fn sync_skip(&mut self) -> bool {
		if let Some(lock) = &mut self.lock {
			mem::replace(&mut lock.skip, self.skip) != self.skip
		} else {
			false
		}
	}

	fn content_unchanged(&self, url: &Url, cha: &Cha) -> bool {
		let Some(lock) = &self.lock else {
			return false;
		};
		let Some(cha_) = &lock.cha else {
			return false;
		};

		*url == lock.url
			&& self.skip == lock.skip
			&& cha.len == cha_.len
			&& cha.modified == cha_.modified
			&& cha.kind == cha_.kind
			&& {
				#[cfg(unix)]
				{
					cha.permissions == cha_.permissions
				}
				#[cfg(windows)]
				{
					true
				}
			}
	}
}

impl PreviewLock {
	#[inline]
	pub fn is_image(&self) -> bool { matches!(self.data, PreviewData::Image) }

	#[inline]
	pub fn is_folder(&self) -> bool { matches!(self.data, PreviewData::Folder) }
}
