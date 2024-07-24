use std::time::{Duration, SystemTime};

use tokio::{pin, task::JoinHandle};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tokio_util::sync::CancellationToken;
use yazi_adapter::ADAPTOR;
use yazi_config::PLUGIN;
use yazi_plugin::{external::Highlighter, isolate, utils::PreviewLock};
use yazi_shared::{fs::{Cha, File, FilesOp, Url}, MIME_DIR};

use crate::folder::Files;

#[derive(Default)]
pub struct Preview {
	pub lock: Option<PreviewLock>,
	pub skip: usize,

	previewer_ct:  Option<CancellationToken>,
	folder_loader: Option<(Url, JoinHandle<()>)>,
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
			isolate::peek_sync(&previewer.run, file, self.skip);
		} else {
			self.previewer_ct = Some(isolate::peek(&previewer.run, file, self.skip));
		}
	}

	pub fn go_folder(&mut self, file: File, mtime: Option<SystemTime>, force: bool) {
		if !force && self.content_unchanged(&file.url, &file.cha) {
			return;
		}

		let url = file.url();
		self.go(file, MIME_DIR, force);

		self.folder_loader.take().map(|(_, h)| h.abort());
		self.folder_loader = Some((
			url.clone(),
			tokio::spawn(async move {
				let Some(meta) = Files::assert_stale(&url, mtime).await else { return };
				let Ok(rx) = Files::from_dir(&url).await else { return };

				let stream =
					UnboundedReceiverStream::new(rx).chunks_timeout(50000, Duration::from_millis(500));
				pin!(stream);

				let ticket = FilesOp::prepare(&url);
				while let Some(chunk) = stream.next().await {
					FilesOp::Part(url.clone(), chunk, ticket).emit();
				}
				FilesOp::Done(url, meta.modified().ok(), ticket).emit();
			}),
		));
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
			&& cha.mtime == lock.cha.mtime
			&& cha.kind == lock.cha.kind
			&& {
				#[cfg(unix)]
				{
					cha.perm == lock.cha.perm
				}
				#[cfg(windows)]
				{
					true
				}
			}
	}
}
