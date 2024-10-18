use std::borrow::Cow;

use tokio_util::sync::CancellationToken;
use yazi_config::PLUGIN;
use yazi_macro::render;
use yazi_plugin::{isolate, utils::SpotLock};
use yazi_shared::fs::{File, Url};

#[derive(Default)]
pub struct Spot {
	pub lock: Option<SpotLock>,
	pub skip: usize,

	ct: Option<CancellationToken>,
}

impl Spot {
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
			self.ct = Some(isolate::peek(&previewer.run, file, mime, self.skip));
		}
	}

	#[inline]
	pub fn abort(&mut self) { self.ct.take().map(|h| h.cancel()); }

	#[inline]
	pub fn reset(&mut self) {
		self.abort();
		render!(self.lock.take().is_some())
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
