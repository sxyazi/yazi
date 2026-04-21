use std::{borrow::Cow, ops::Deref, sync::Arc};

use serde::Deserialize;
use yazi_fs::File;
use yazi_shared::{Id, event::Cmd};

use crate::{Mixable, Pattern, Priority, Selectable, Selector, plugin::{Preloaders, preloader_id}};

#[derive(Debug, Deserialize)]
pub struct Preloader {
	#[serde(skip, default = "preloader_id")]
	pub id:       Id,
	#[serde(skip)]
	pub idx:      u8,
	#[serde(flatten)]
	pub selector: Selector,
	pub run:      Cmd,
	#[serde(default)]
	pub next:     bool,
	#[serde(default)]
	pub prio:     Priority,
}

impl Deref for Preloader {
	type Target = Cmd;

	fn deref(&self) -> &Self::Target { &self.run }
}

impl Selectable for Preloader {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }
}

impl Mixable for Preloader {}

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
