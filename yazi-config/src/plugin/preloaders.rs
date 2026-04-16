use std::{borrow::Cow, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use serde::Deserialize;
use yazi_fs::File;
use yazi_shared::Id;
use yazi_shim::arc_swap::IntoPointee;

use super::Preloader;
use crate::Selectable;

#[derive(Debug, Default, Deserialize)]
pub struct Preloaders(ArcSwap<Vec<Arc<Preloader>>>);

impl Deref for Preloaders {
	type Target = ArcSwap<Vec<Arc<Preloader>>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<Arc<Preloader>>> for Preloaders {
	fn from(inner: Vec<Arc<Preloader>>) -> Self { Self(inner.into_pointee()) }
}

impl From<Arc<Vec<Arc<Preloader>>>> for Preloaders {
	fn from(inner: Arc<Vec<Arc<Preloader>>>) -> Self { Self(inner.into()) }
}

impl Preloaders {
	pub fn matches<'a>(&self, file: &'a File, mime: &'a str) -> PreloaderMatcher<'a> {
		self.matcher(Some(file), Some(mime))
	}

	pub fn matcher<'a, F, M>(&self, file: Option<F>, mime: Option<M>) -> PreloaderMatcher<'a>
	where
		F: Into<Cow<'a, File>>,
		M: Into<Cow<'a, str>>,
	{
		PreloaderMatcher {
			preloaders: self.load_full(),
			file: file.map(Into::into),
			mime: mime.map(Into::into),
			..Default::default()
		}
	}

	pub(crate) fn unwrap_unchecked(self) -> Vec<Arc<Preloader>> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique preloaders arc")
	}
}

// --- Matcher
#[derive(Default)]
pub struct PreloaderMatcher<'a> {
	pub preloaders: Arc<Vec<Arc<Preloader>>>,
	pub id:         Id,
	pub file:       Option<Cow<'a, File>>,
	pub mime:       Option<Cow<'a, str>>,
	pub all:        bool,
	pub offset:     usize,
	pub stop:       bool,
}

impl From<&Preloaders> for PreloaderMatcher<'_> {
	fn from(preloaders: &Preloaders) -> Self {
		Self { preloaders: preloaders.load_full(), all: true, ..Default::default() }
	}
}

impl PreloaderMatcher<'_> {
	pub fn matches(&self, preloader: &Preloader) -> bool {
		if self.all {
			true
		} else if self.id != Id::ZERO {
			preloader.id == self.id
		} else {
			preloader.match_with(self.file.as_deref(), self.mime.as_deref())
		}
	}
}

impl Iterator for PreloaderMatcher<'_> {
	type Item = Arc<Preloader>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.stop && !self.all {
			return None;
		}

		while let Some(preloader) = self.preloaders.get(self.offset) {
			self.offset += 1;
			if self.matches(preloader) {
				self.stop = !preloader.next;
				return Some(preloader.clone());
			}
		}
		None
	}
}
