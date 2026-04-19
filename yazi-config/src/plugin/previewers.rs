use std::{borrow::Cow, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use serde::Deserialize;
use yazi_fs::File;
use yazi_shim::{arc_swap::{ArcSwapExt, IntoPointee}, vec::{IndexAtError, VecExt}};

use super::Previewer;
use crate::{mix, plugin::PreviewerMatcher};

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
		self.0.try_rcu(|previewers| {
			let i = previewers.index_at(index)?;
			if i == previewers.len() {
				Ok(mix(Vec::<Previewer>::new(), previewers.iter().cloned(), [previewer.clone()]))
			} else {
				let (before, after) = previewers.split_at(i);
				Ok(mix(
					Vec::<Previewer>::new(),
					before.iter().cloned().chain([previewer.clone()]).chain(after.iter().cloned()),
					Vec::<Previewer>::new(),
				))
			}
		})?;

		Ok(())
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
