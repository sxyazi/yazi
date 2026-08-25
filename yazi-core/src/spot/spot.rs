use yazi_binding::Scope;
use yazi_config::YAZI;
use yazi_fs::file::File;
use yazi_macro::render;
use yazi_runner::RUNNER;
use yazi_shared::{pool::Symbol, url::UrlBuf};

use crate::spot::SpotLock;

#[derive(Default)]
pub struct Spot {
	pub lock: Option<SpotLock>,
	pub skip: usize,

	scope: Scope,
}

impl Spot {
	pub fn go(&mut self, file: File, mime: Symbol<str>, force: bool) {
		if mime.is_empty() {
			return; // Wait till mimetype is resolved to avoid flickering
		} else if !force && self.same_lock(&file, &mime) {
			return;
		}

		let Some(spotter) = YAZI.plugin.spotters.matches(&file, &mime) else {
			return self.reset();
		};

		self.abort();
		self.scope = RUNNER.spot(spotter, file, mime, self.skip);
	}

	pub fn visible(&self) -> bool { self.lock.is_some() }

	fn abort(&mut self) { self.scope.take().cancel(); }

	pub fn reset(&mut self) {
		self.abort();
		render!(self.lock.take().is_some());
	}

	pub fn same_url(&self, url: &UrlBuf) -> bool { self.lock.as_ref().is_some_and(|l| *url == l.url) }

	fn same_file(&self, file: &File, mime: &str) -> bool {
		self.same_url(&file.url)
			&& self.lock.as_ref().is_some_and(|l| file.cha.hits(l.cha) && mime == l.mime)
	}

	fn same_lock(&self, file: &File, mime: &str) -> bool {
		self.same_file(file, mime) && self.lock.as_ref().is_some_and(|l| self.skip == l.skip)
	}
}
