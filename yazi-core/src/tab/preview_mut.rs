use std::ops::{Deref, DerefMut};

use tokio::select;
use yazi_adapter::ADAPTOR;
use yazi_config::YAZI;
use yazi_fs::{FsHash64, file::File};
use yazi_macro::render;
use yazi_runner::{RUNNER, previewer::{PeekError, PeekJob}};
use yazi_shared::{pool::Symbol, scope::Scope};

use super::{Preview, PreviewSig, Tab};
use crate::{AppProxy, MgrProxy};

pub struct PreviewMut<'a> {
	inner:  &'a mut Preview,
	parent: &'a Scope,
}

impl Deref for PreviewMut<'_> {
	type Target = Preview;

	fn deref(&self) -> &Self::Target { self.inner }
}

impl DerefMut for PreviewMut<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target { self.inner }
}

impl<'a> From<&'a mut Tab> for PreviewMut<'a> {
	fn from(tab: &'a mut Tab) -> Self { Self { inner: &mut tab.preview, parent: &tab.scope } }
}

impl PreviewMut<'_> {
	fn renew(&mut self) {
		self.scope.cancel();
		self.scope = self.parent.child();
	}

	pub fn reset(&mut self) {
		self.reset_image();
		render!(self.lock.take().is_some());
	}

	pub fn reset_image(&mut self) {
		self.renew();
		ADAPTOR.image_hide().ok();
	}

	pub fn go(mut self, file: File, mime: Symbol<str>, force: bool) {
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

		self.renew();
		let job = PeekJob { previewer, file, mime, sig, skip: self.skip, scope: self.scope.clone() };

		tokio::spawn(async move {
			let result = select! {
				biased;
				_ = job.scope.cancelled() => Err(PeekError::Cancelled),
				r = RUNNER.peek(&job) => r,
			};

			match result {
				Ok(()) | Err(PeekError::Cancelled) => {}
				Err(PeekError::ShouldSync) => AppProxy::plugin_peek(job),
				Err(e) => MgrProxy::update_peeked_error(job, e.to_string()),
			}
		});
	}
}
