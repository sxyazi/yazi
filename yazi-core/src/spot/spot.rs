use std::borrow::Cow;

use tokio_util::sync::CancellationToken;
use yazi_config::YAZI;
use yazi_fs::File;
use yazi_macro::render;
use yazi_plugin::{isolate, utils::SpotLock};
use yazi_shared::url::Url;

#[derive(Default)]
pub struct Spot {
	pub lock: Option<SpotLock>,
	pub skip: usize,

	pub(super) ct: Option<CancellationToken>,
}

impl Spot {
	pub fn go(&mut self, file: File, mime: Cow<'static, str>) {
		if mime.is_empty() {
			return; // Wait till mimetype is resolved to avoid flickering
		} else if self.same_lock(&file, &mime) {
			return;
		}

		let Some(spotter) = YAZI.plugin.spotter(&file.url, &mime) else {
			return self.close(());
		};

		self.abort();
		self.ct = Some(isolate::spot(&spotter.run, file, mime, self.skip));
	}

	#[inline]
	pub fn visible(&self) -> bool { self.lock.is_some() }

	#[inline]
	pub fn abort(&mut self) { self.ct.take().map(|ct| ct.cancel()); }

	#[inline]
	pub fn reset(&mut self) {
		self.abort();
		render!(self.lock.take().is_some());
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
