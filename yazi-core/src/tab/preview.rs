use std::{borrow::Cow, ops::Not, time::Duration};

use tokio::{pin, task::JoinHandle};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tokio_util::sync::CancellationToken;
use yazi_adapter::ADAPTOR;
use yazi_config::PLUGIN;
use yazi_fs::{Cha, File, Files, FilesOp};
use yazi_macro::render;
use yazi_plugin::{external::Highlighter, isolate, utils::PreviewLock};
use yazi_shared::{MIME_DIR, url::Url};

#[derive(Default)]
pub struct Preview {
	pub lock: Option<PreviewLock>,
	pub skip: usize,

	previewer_ct:  Option<CancellationToken>,
	folder_loader: Option<JoinHandle<()>>,
}

impl Preview {
	pub fn go(&mut self, file: File, mime: Cow<'static, str>, force: bool) {
		if mime.is_empty() {
			return; // Wait till mimetype is resolved to avoid flickering
		} else if !force && self.same_lock(&file, &mime) {
			return;
		}

		let Some(previewer) = PLUGIN.previewer(&file.url, &mime) else {
			return self.reset();
		};

		self.abort();
		if previewer.sync {
			isolate::peek_sync(&previewer.run, file, mime, self.skip);
		} else {
			self.previewer_ct = Some(isolate::peek(&previewer.run, file, mime, self.skip));
		}
	}

	pub fn go_folder(&mut self, file: File, dir: Option<Cha>, force: bool) {
		let cwd = self.same_file(&file, MIME_DIR).not().then(|| file.url_owned());
		self.go(file, Cow::Borrowed(MIME_DIR), force);

		let Some(cwd) = cwd else { return };
		self.folder_loader.take().map(|h| h.abort());
		self.folder_loader = Some(tokio::spawn(async move {
			let Some(new) = Files::assert_stale(&cwd, dir.unwrap_or(Cha::dummy())).await else {
				return;
			};
			let Ok(rx) = Files::from_dir(&cwd).await else { return };

			let stream =
				UnboundedReceiverStream::new(rx).chunks_timeout(50000, Duration::from_millis(500));
			pin!(stream);

			let ticket = FilesOp::prepare(&cwd);
			while let Some(chunk) = stream.next().await {
				FilesOp::Part(cwd.clone(), chunk, ticket).emit();
			}
			FilesOp::Done(cwd, new, ticket).emit();
		}));
	}

	#[inline]
	pub fn abort(&mut self) {
		self.previewer_ct.take().map(|ct| ct.cancel());
		Highlighter::abort();
	}

	#[inline]
	pub fn reset(&mut self) {
		self.abort();
		ADAPTOR.image_hide().ok();
		render!(self.lock.take().is_some())
	}

	#[inline]
	pub fn reset_image(&mut self) {
		self.abort();
		ADAPTOR.image_hide().ok();
	}

	#[inline]
	pub fn same_url(&self, url: &Url) -> bool { self.lock.as_ref().is_some_and(|l| *url == l.url) }

	#[inline]
	pub fn same_file(&self, file: &File, mime: &str) -> bool {
		self.same_url(&file.url)
			&& self.lock.as_ref().is_some_and(|l| file.cha.hits(l.cha) && mime == l.mime)
	}

	#[inline]
	pub fn same_lock(&self, file: &File, mime: &str) -> bool {
		self.same_file(file, mime) && self.lock.as_ref().is_some_and(|l| self.skip == l.skip)
	}
}
