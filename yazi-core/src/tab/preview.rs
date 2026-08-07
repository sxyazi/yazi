use tokio::task::JoinHandle;
use yazi_adapter::ADAPTOR;
use yazi_binding::Scope;
use yazi_config::YAZI;
use yazi_fs::{FsHash64, file::File};
use yazi_macro::render;
use yazi_runner::{RUNNER, previewer::{PeekError, PeekJob}};
use yazi_shared::{id::Id, pool::Symbol, url::UrlBuf};

use crate::{AppProxy, Highlighter, MgrProxy, tab::{PreviewLock, PreviewSig}};

#[derive(Default)]
pub struct Preview {
	pub lock: Option<PreviewLock>,
	pub skip: usize,

	handle: Option<JoinHandle<()>>,
	scope:  Scope,
}

impl Preview {
	pub fn go(&mut self, file: File, mime: Symbol<str>, force: bool) {
		if mime.is_empty() {
			return; // Wait till mimetype is resolved to avoid flickering
		}

		let sig = PreviewSig::new(&file, &mime).hash_id();
		if !force && self.same_lock(sig) {
			return;
		}

		let Some(previewer) = YAZI.plugin.previewers.matches(&file, &mime) else {
			return self.reset();
		};

		self.abort();
		self.scope = Scope::new();

		let job = PeekJob { previewer, file, mime, sig, skip: self.skip };
		let scope = self.scope.clone();

		self.handle = Some(tokio::spawn(async move {
			let mut rx = RUNNER.peek(&job).await;
			match rx.recv().await.unwrap_or(Err(PeekError::Cancelled)) {
				Ok(()) | Err(PeekError::Cancelled) => {}
				Err(PeekError::ShouldSync) => AppProxy::plugin_peek(job, scope),
				Err(e) => MgrProxy::update_peeked_error(job, e.to_string(), scope),
			}
		}));
	}

	pub fn abort(&mut self) {
		self.handle.take().map(|ct| ct.abort());
		self.scope.take().cancel();
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
			&& matches!(&self.lock, Some(l) if l.sig == PreviewSig::new(file, mime).hash_id())
	}

	fn same_lock(&self, sig: Id) -> bool {
		self.lock.as_ref().is_some_and(|l| l.sig == sig && l.skip == self.skip)
	}
}
