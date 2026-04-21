use std::{borrow::Cow, ops::Deref, sync::Arc};

use hashbrown::HashSet;
use serde::Deserialize;
use yazi_fs::File;
use yazi_shared::{Id, event::Cmd};

use crate::{Mixable, Pattern, Priority, Selectable, Selector, plugin::{Fetchers, fetcher_id}};

#[derive(Debug, Deserialize)]
pub struct Fetcher {
	#[serde(skip, default = "fetcher_id")]
	pub id:       Id,
	#[serde(skip)]
	pub idx:      u8,
	#[serde(flatten)]
	pub selector: Selector,
	pub run:      Cmd,
	#[serde(default)]
	pub prio:     Priority,
	pub group:    String,
}

impl Deref for Fetcher {
	type Target = Cmd;

	fn deref(&self) -> &Self::Target { &self.run }
}

impl Selectable for Fetcher {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }
}

impl Mixable for Fetcher {}

// --- Matcher
#[derive(Default)]
pub struct FetcherMatcher<'a> {
	pub fetchers: Arc<Vec<Arc<Fetcher>>>,
	pub id:       Id,
	pub file:     Option<Cow<'a, File>>,
	pub mime:     Option<Cow<'a, str>>,
	pub all:      bool,
	pub offset:   usize,
	pub seen:     HashSet<String>,
}

impl From<&Fetchers> for FetcherMatcher<'_> {
	fn from(fetchers: &Fetchers) -> Self {
		Self { fetchers: fetchers.load_full(), all: true, ..Default::default() }
	}
}

impl FetcherMatcher<'_> {
	pub fn matches(&self, fetcher: &Fetcher) -> bool {
		if self.all {
			true
		} else if self.id != Id::ZERO {
			fetcher.id == self.id
		} else {
			fetcher.match_with(self.file.as_deref(), self.mime.as_deref())
		}
	}
}

impl Iterator for FetcherMatcher<'_> {
	type Item = Arc<Fetcher>;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(fetcher) = self.fetchers.get(self.offset) {
			self.offset += 1;
			if !self.matches(fetcher) {
				continue;
			}
			if self.all || self.seen.insert(fetcher.group.clone()) {
				return Some(fetcher.clone());
			}
		}
		None
	}
}
