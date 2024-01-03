use std::time::Duration;

use tokio::{pin, task::JoinHandle};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tokio_util::sync::CancellationToken;
use yazi_adaptor::ADAPTOR;
use yazi_config::PLUGIN;
use yazi_plugin::{external::Highlighter, isolate, utils::PreviewLock};
use yazi_shared::{fs::{Cha, File, FilesOp, Url}, MIME_DIR};

use crate::folder::Files;

#[derive(Default)]
pub struct Preview {
	pub lock: Option<PreviewLock>,
	pub skip: usize,

	previewer_ct:  Option<CancellationToken>,
	folder_handle: Option<JoinHandle<()>>,
}

impl Preview {
	pub fn go(&mut self, file: File, mime: &str, force: bool) {
		if !force && self.content_unchanged(&file.url, &file.cha) {
			return;
		}

		let Some(previewer) = PLUGIN.previewer(&file.url, mime) else {
			self.reset();
			return;
		};

		self.abort();
		if previewer.sync {
			isolate::peek_sync(&previewer.exec, file, self.skip);
		} else {
			self.previewer_ct = Some(isolate::peek(&previewer.exec, file, self.skip));
		}
	}

	pub fn go_folder(&mut self, file: File, force: bool) {
		if !force && self.content_unchanged(&file.url, &file.cha) {
			return;
		}

		self.go(file.clone(), MIME_DIR, force);

		self.folder_handle.take().map(|h| h.abort());
		self.folder_handle = Some(tokio::spawn(async move {
			let Ok(rx) = Files::from_dir(&file.url).await else {
				file.url.parent_url().map(|p| FilesOp::Deleting(p, vec![file.url]).emit());
				return;
			};

			let stream =
				UnboundedReceiverStream::new(rx).chunks_timeout(10000, Duration::from_millis(350));
			pin!(stream);

			let ticket = FilesOp::prepare(&file.url);
			while let Some(chunk) = stream.next().await {
				FilesOp::Part(file.url.clone(), chunk, ticket).emit();
			}
		}));
	}

	#[inline]
	pub fn abort(&mut self) {
		self.previewer_ct.take().map(|ct| ct.cancel());
		Highlighter::abort();
	}

	#[inline]
	pub fn reset(&mut self) -> bool {
		self.abort();
		ADAPTOR.image_hide().ok();
		self.lock.take().is_some()
	}

	#[inline]
	pub fn reset_image(&mut self) {
		self.abort();
		ADAPTOR.image_hide().ok();
	}

	#[inline]
	pub fn same_url(&self, url: &Url) -> bool {
		matches!(self.lock, Some(ref lock) if lock.url == *url)
	}

	fn content_unchanged(&self, url: &Url, cha: &Cha) -> bool {
		let Some(lock) = &self.lock else {
			return false;
		};

		*url == lock.url
			&& self.skip == lock.skip
			&& cha.len == lock.cha.len
			&& cha.modified == lock.cha.modified
			&& cha.kind == lock.cha.kind
			&& {
				#[cfg(unix)]
				{
					cha.permissions == lock.cha.permissions
				}
				#[cfg(windows)]
				{
					true
				}
			}
	}
}
