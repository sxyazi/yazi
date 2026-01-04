use std::time::Duration;

use tokio::{pin, task::JoinHandle};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tokio_util::sync::CancellationToken;
use yazi_adapter::ADAPTOR;
use yazi_config::{LAYOUT, YAZI};
use yazi_fs::{File, Files, FilesOp, cha::Cha};
use yazi_macro::render;
use yazi_parser::mgr::PreviewLock;
use yazi_plugin::{external::Highlighter, isolate};
use yazi_shared::{pool::Symbol, url::{UrlBuf, UrlLike}};
use yazi_vfs::{VfsFiles, VfsFilesOp};

#[derive(Default)]
pub struct Preview {
	pub lock: Option<PreviewLock>,
	pub skip: usize,

	previewer_ct:    Option<CancellationToken>,
	pub folder_lock: Option<UrlBuf>,
	folder_loader:   Option<JoinHandle<()>>,
}

impl Preview {
	pub fn go(&mut self, file: File, mime: Symbol<str>, force: bool) {
		if mime.is_empty() {
			return; // Wait till mimetype is resolved to avoid flickering
		} else if !force && self.same_lock(&file, &mime) {
			return;
		}

		let Some(previewer) = YAZI.plugin.previewer(&file, &mime) else {
			return self.reset();
		};

		self.abort();
		self.previewer_ct = isolate::peek(&previewer.run, file, mime, self.skip);
	}

	pub fn go_folder(&mut self, file: File, dir: Option<Cha>, mime: Symbol<str>, force: bool) {
		if !file.url.is_internal() {
			return self.go(file, mime, force);
		} else if self.folder_lock.as_ref() == Some(&file.url) {
			return self.go(file, mime, force);
		}

		let wd = file.url_owned();
		self.go(file, mime, force);

		self.folder_lock = Some(wd.clone());
		self.folder_loader.take().map(|h| h.abort());
		self.folder_loader = Some(tokio::spawn(async move {
			let Some(new) = Files::assert_stale(&wd, dir.unwrap_or_default()).await else { return };

			let rx = match Files::from_dir(&wd).await {
				Ok(rx) => rx,
				Err(e) => return FilesOp::issue_error(&wd, e).await,
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

	pub fn abort(&mut self) {
		self.previewer_ct.take().map(|ct| ct.cancel());
		Highlighter::abort();
	}

	pub fn reset(&mut self) {
		self.abort();
		ADAPTOR.get().image_hide().ok();
		render!(self.lock.take().is_some())
	}

	pub fn reset_image(&mut self) {
		self.abort();
		ADAPTOR.get().image_hide().ok();
	}

	pub fn same_url(&self, url: &UrlBuf) -> bool { matches!(&self.lock, Some(l) if l.url == *url) }

	pub fn same_file(&self, file: &File, mime: &str) -> bool {
		self.same_url(&file.url)
			&& matches!(&self.lock , Some(l) if l.cha.hits(file.cha) && l.mime == mime && *l.area == LAYOUT.get().preview)
	}

	pub fn same_lock(&self, file: &File, mime: &str) -> bool {
		self.same_file(file, mime) && matches!(&self.lock, Some(l) if l.skip == self.skip)
	}

	pub fn same_folder(&self, url: &UrlBuf) -> bool { self.folder_lock.as_ref() == Some(url) }
}
