use std::{borrow::Cow, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use serde::Deserialize;
use yazi_fs::File;
use yazi_shared::Id;
use yazi_shim::{arc_swap::IntoPointee, vec::{IndexAtError, VecExt}};

use super::Previewer;
use crate::{Selectable, mix};

#[derive(Debug, Default, Deserialize)]
pub struct Previewers(ArcSwap<Vec<Arc<Previewer>>>);

impl Deref for Previewers {
	type Target = ArcSwap<Vec<Arc<Previewer>>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<Arc<Previewer>>> for Previewers {
	fn from(inner: Vec<Arc<Previewer>>) -> Self { Self(inner.into_pointee()) }
}

impl Previewers {
	pub fn matches(&self, file: &File, mime: &str) -> Option<Arc<Previewer>> {
		self.matcher(Some(file), Some(mime)).next()
	}

	pub fn matcher<'a, F, M>(&self, file: Option<F>, mime: Option<M>) -> PreviewerMatcher<'a>
	where
		F: Into<Cow<'a, File>>,
		M: Into<Cow<'a, str>>,
	{
		PreviewerMatcher {
			previewers: self.0.load_full(),
			file: file.map(Into::into),
			mime: mime.map(Into::into),
			..Default::default()
		}
	}

	pub fn insert(&self, index: isize, previewer: Arc<Previewer>) -> Result<(), IndexAtError> {
		let mut err = None;

		self.0.rcu(|previewers| match previewers.index_at(index) {
			Ok(n) if n == previewers.len() => {
				mix(Vec::<Previewer>::new(), previewers.iter().cloned(), [previewer.clone()])
			}
			Ok(n) => {
				let (before, after) = previewers.split_at(n);
				mix(
					Vec::<Previewer>::new(),
					before.iter().cloned().chain([previewer.clone()]).chain(after.iter().cloned()),
					Vec::<Previewer>::new(),
				)
			}
			Err(e) => {
				err = Some(e);
				Vec::clone(previewers)
			}
		});

		err.map_or(Ok(()), Err)
	}

	pub fn remove(&self, matcher: PreviewerMatcher) {
		self.0.rcu(|previewers| {
			let mut next = Vec::clone(previewers);
			next.retain(|previewer| !matcher.matches(previewer));
			next
		});
	}

	pub(crate) fn unwrap_unchecked(self) -> Vec<Arc<Previewer>> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique previewers arc")
	}
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
