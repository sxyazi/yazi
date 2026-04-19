use std::{borrow::Cow, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use serde::Deserialize;
use yazi_fs::File;
use yazi_shared::Id;
use yazi_shim::arc_swap::IntoPointee;

use super::Spotter;
use crate::plugin::SpotterMatcher;

#[derive(Debug, Default, Deserialize)]
pub struct Spotters(ArcSwap<Vec<Arc<Spotter>>>);

impl Deref for Spotters {
	type Target = ArcSwap<Vec<Arc<Spotter>>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<Arc<Spotter>>> for Spotters {
	fn from(inner: Vec<Arc<Spotter>>) -> Self { Self(inner.into_pointee()) }
}

impl Spotters {
	pub fn matches(&self, file: &File, mime: &str) -> Option<Arc<Spotter>> {
		self.matcher(Some(file), Some(mime)).next()
	}

	pub fn matcher<'a, F, M>(&self, file: Option<F>, mime: Option<M>) -> SpotterMatcher<'a>
	where
		F: Into<Cow<'a, File>>,
		M: Into<Cow<'a, str>>,
	{
		SpotterMatcher {
			spotters: self.load_full(),
			id: Id::ZERO,
			file: file.map(Into::into),
			mime: mime.map(Into::into),
			..Default::default()
		}
	}

	pub(crate) fn unwrap_unchecked(self) -> Vec<Arc<Spotter>> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique spotters arc")
	}
}
