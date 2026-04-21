use std::{borrow::Cow, ops::Deref, sync::Arc};

use serde::Deserialize;
use yazi_fs::File;
use yazi_shared::{Id, event::Cmd};

use crate::{Mixable, Pattern, Selectable, Selector, plugin::{Spotters, spotter_id}};

#[derive(Debug, Deserialize)]
pub struct Spotter {
	#[serde(skip, default = "spotter_id")]
	pub id:       Id,
	#[serde(flatten)]
	pub selector: Selector,
	pub run:      Cmd,
}

impl Deref for Spotter {
	type Target = Cmd;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.run }
}

impl Selectable for Spotter {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }
}

impl Mixable for Spotter {
	fn any_file(&self) -> bool { self.selector.any_file() }

	fn any_dir(&self) -> bool { self.selector.any_dir() }
}

// --- Matcher
#[derive(Default)]
pub struct SpotterMatcher<'a> {
	pub spotters: Arc<Vec<Arc<Spotter>>>,
	pub id:       Id,
	pub file:     Option<Cow<'a, File>>,
	pub mime:     Option<Cow<'a, str>>,
	pub all:      bool,
	pub offset:   usize,
}

impl From<&Spotters> for SpotterMatcher<'_> {
	fn from(spotters: &Spotters) -> Self {
		Self { spotters: spotters.load_full(), all: true, ..Default::default() }
	}
}

impl SpotterMatcher<'_> {
	pub fn matches(&self, spotter: &Spotter) -> bool {
		if self.all {
			true
		} else if self.id != Id::ZERO {
			spotter.id == self.id
		} else {
			spotter.match_with(self.file.as_deref(), self.mime.as_deref())
		}
	}
}

impl Iterator for SpotterMatcher<'_> {
	type Item = Arc<Spotter>;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(spotter) = self.spotters.get(self.offset) {
			self.offset += 1;
			if self.matches(spotter) {
				return Some(spotter.clone());
			}
		}
		None
	}
}
