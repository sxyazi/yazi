use std::{borrow::Cow, ops::Deref, sync::Arc};

use serde::Deserialize;
use yazi_fs::File;
use yazi_shared::{Id, event::Cmd};

use crate::{Mixable, Pattern, Selectable, Selector, plugin::{Previewers, previewer_id}};

#[derive(Debug, Deserialize)]
pub struct Previewer {
	#[serde(skip, default = "previewer_id")]
	pub id:       Id,
	#[serde(flatten)]
	pub selector: Selector,
	pub run:      Cmd,
}

impl Deref for Previewer {
	type Target = Cmd;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.run }
}

impl Selectable for Previewer {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }
}

impl Mixable for Previewer {
	fn any_file(&self) -> bool { self.selector.any_file() }

	fn any_dir(&self) -> bool { self.selector.any_dir() }
}

// --- Matcher
#[derive(Default)]
pub struct PreviewerMatcher<'a> {
	pub previewers: Arc<Vec<Arc<Previewer>>>,
	pub id:         Id,
	pub file:       Option<Cow<'a, File>>,
	pub mime:       Option<Cow<'a, str>>,
	pub all:        bool,
	pub offset:     usize,
}

impl From<&Previewers> for PreviewerMatcher<'_> {
	fn from(previewers: &Previewers) -> Self {
		Self { previewers: previewers.load_full(), all: true, ..Default::default() }
	}
}

impl PreviewerMatcher<'_> {
	pub fn matches(&self, previewer: &Previewer) -> bool {
		if self.all {
			true
		} else if self.id != Id::ZERO {
			previewer.id == self.id
		} else {
			previewer.match_with(self.file.as_deref(), self.mime.as_deref())
		}
	}
}

impl Iterator for PreviewerMatcher<'_> {
	type Item = Arc<Previewer>;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(previewer) = self.previewers.get(self.offset) {
			self.offset += 1;
			if self.matches(previewer) {
				return Some(previewer.clone());
			}
		}
		None
	}
}
