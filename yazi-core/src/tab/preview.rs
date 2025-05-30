use std::{borrow::Cow, time::Duration};

use tokio::{pin, task::JoinHandle};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tokio_util::sync::CancellationToken;
use yazi_adapter::ADAPTOR;
use yazi_config::YAZI;
use yazi_fs::{File, Files, FilesOp, cha::Cha};
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

		let Some(previewer) = YAZI.plugin.previewer(&file.url, &mime) else {
			return self.reset();
		};

		self.abort();
		self.previewer_ct = isolate::peek(&previewer.run, file, mime, self.skip);
	}

	pub fn go_folder(&mut self, file: File, dir: Option<Cha>, force: bool) {
		let same = self.same_file(&file, MIME_DIR);
		let (wd, cha) = (file.url_owned(), file.cha);

		self.go(file, Cow::Borrowed(MIME_DIR), force);
		if same {
			return;
		}

		self.lock =
			Some(PreviewLock { url: wd.clone(), cha, mime: MIME_DIR.to_owned(), ..Default::default() });
		self.folder_loader.take().map(|h| h.abort());
		self.folder_loader = Some(tokio::spawn(async move {
			let Some(new) = Files::assert_stale(&wd, dir.unwrap_or_default()).await else { return };

			let rx = match Files::from_dir(&wd).await {
				Ok(rx) => rx,
				Err(e) => return FilesOp::issue_error(&wd, e.kind()).await,
			};

			let stream =
				UnboundedReceiverStream::new(rx).chunks_timeout(50000, Duration::from_millis(500));
			pin!(stream);

			let ticket = FilesOp::prepare(&wd);
			while let Some(chunk) = stream.next().await {
				FilesOp::Part(wd.clone(), chunk, ticket).emit();
			}
			FilesOp::Done(wd, new, ticket).emit();
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
		ADAPTOR.get().image_hide().ok();
		render!(self.lock.take().is_some())
	}

	#[inline]
	pub fn reset_image(&mut self) {
		self.abort();
		ADAPTOR.get().image_hide().ok();
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
