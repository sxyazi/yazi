use tokio::task::JoinHandle;
use yazi_adapter::ADAPTOR;
use yazi_config::{LAYOUT, YAZI};
use yazi_fs::file::File;
use yazi_macro::render;
use yazi_runner::{RUNNER, previewer::{PeekError, PeekJob}};
use yazi_shared::{pool::Symbol, url::UrlBuf};

use crate::{AppProxy, Highlighter, MgrProxy, tab::PreviewLock};

#[derive(Default)]
pub struct Preview {
	pub lock: Option<PreviewLock>,
	pub skip: usize,

	handle: Option<JoinHandle<()>>,
}

impl Preview {
	pub fn go(&mut self, file: File, mime: Symbol<str>, force: bool) {
		if mime.is_empty() {
			return; // Wait till mimetype is resolved to avoid flickering
		} else if !force && self.same_lock(&file, &mime) {
			return;
		}

		let Some(previewer) = YAZI.plugin.previewers.matches(&file, &mime) else {
			return self.reset();
		};

		self.abort();
		let job = PeekJob { previewer, file, mime, skip: self.skip };

		self.handle = Some(tokio::spawn(async move {
			let mut rx = RUNNER.peek(&job).await;
			match rx.recv().await.unwrap_or(Err(PeekError::Cancelled)) {
				Ok(()) | Err(PeekError::Cancelled) => {}
				Err(PeekError::ShouldSync) => AppProxy::plugin_peek(job),
				Err(e) => MgrProxy::update_peeked_error(job, e.to_string()),
			}
		}));
	}

	pub fn abort(&mut self) {
		self.handle.take().map(|ct| ct.abort());
		Highlighter::abort();
	}

	pub fn reset(&mut self) {
		self.abort();
		ADAPTOR.image_hide().ok();
		render!(self.lock.take().is_some())
	}

	pub fn reset_image(&mut self) {
		self.abort();
		ADAPTOR.image_hide().ok();
	}

	pub fn same_url(&self, url: &UrlBuf) -> bool { matches!(&self.lock, Some(l) if l.url == *url) }

	pub fn same_file(&self, file: &File, mime: &str) -> bool {
		self.same_url(&file.url)
			&& matches!(&self.lock , Some(l) if l.cha.hits(file.cha) && l.mime == mime && *l.area == LAYOUT.get().preview)
	}

	pub fn same_lock(&self, file: &File, mime: &str) -> bool {
		self.same_file(file, mime) && matches!(&self.lock, Some(l) if l.skip == self.skip)
	}
}
