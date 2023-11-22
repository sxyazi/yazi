use std::{mem, time::Duration};

use tokio::{pin, task::JoinHandle};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_adaptor::ADAPTOR;
use yazi_config::MANAGER;
use yazi_shared::{MimeKind, PeekError, Url};

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
	pub fn go(&mut self, url: &Url, mime: &str) {
		self.reset(|_| true);
		let (url, skip, kind) = (url.clone(), self.skip, MimeKind::new(mime));

		self.handle = Some(tokio::spawn(async move {
			match Provider::auto(kind, &url, skip).await {
				Ok(data) => {
					emit!(Preview(PreviewLock { url, skip, data }));
				}
				Err(PeekError::Exceed(max)) => {
					Manager::_peek_upper_bound(max, &url);
				}
				_ => {}
			}
		}));
	}

	pub fn go_folder(&mut self, url: Url, in_chunks: bool) {
		self.reset(|_| true);
		emit!(Preview(PreviewLock { url: url.clone(), skip: self.skip, data: PreviewData::Folder }));

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

	pub fn reset<F: FnOnce(&PreviewLock) -> bool>(&mut self, f: F) -> bool {
		self.handle.take().map(|h| h.abort());
		Highlighter::abort();
		ADAPTOR.image_hide(MANAGER.layout.image_rect()).ok();

		let Some(ref lock) = self.lock else {
			return false;
		};

		if !f(lock) {
			return false;
		}

		let b = !lock.is_image();
		self.lock = None;
		b
	}
}

impl Preview {
	// --- skip
	#[inline]
	pub fn arrow(&mut self, step: isize, mime: &str) -> bool {
		let size = Provider::step_size(MimeKind::new(mime), step.unsigned_abs());
		let skip = if step < 0 { self.skip.saturating_sub(size) } else { self.skip + size };
		mem::replace(&mut self.skip, skip) != skip
	}

	#[inline]
	pub fn set_skip(&mut self, skip: usize) -> bool { mem::replace(&mut self.skip, skip) != skip }

	#[inline]
	pub fn apply_bound(&mut self, max: usize) -> bool {
		if self.skip <= max {
			return false;
		}

		self.skip = max;
		true
	}
}

impl PreviewLock {
	#[inline]
	pub fn is_image(&self) -> bool { matches!(self.data, PreviewData::Image) }

	#[inline]
	pub fn is_folder(&self) -> bool { matches!(self.data, PreviewData::Folder) }
}
